use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread,
};

use crate::Aggregator;

/// A simple thread pool for parallelizing the 1BRC workload.
///
/// Workers pull chunks from a shared queue, process lines into
/// per-station `Aggregator` maps, and send results back to the
/// coordinator thread via a channel.
struct Pool {
    threads: Vec<std::thread::JoinHandle<()>>,
    /// Shared iterator that yields raw byte chunks for each worker.
    chunk_rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    /// Channel for workers to send their local aggregation results back.
    result_rx: Receiver<HashMap<String, Aggregator>>,
}

impl Pool {
    /// Spawn `worker_count` threads. Each worker repeatedly:
    ///  1. Locks the shared receiver and pops a chunk.
    ///  2. Parses the chunk into station→Aggregator entries.
    ///  3. Sends the local map back through `result_tx`.
    fn new(worker_count: usize, chunk_rx: Receiver<Vec<u8>>) -> Self {
        let chunk_rx = Arc::new(Mutex::new(chunk_rx));
        let (result_tx, result_rx) = mpsc::channel();

        let threads: Vec<_> = (0..worker_count)
            .map(|_| {
                let chunk_rx = Arc::clone(&chunk_rx);
                let result_tx = result_tx.clone();
                std::thread::spawn(move || {
                    // Local aggregation map for this worker.
                    let mut local: HashMap<String, Aggregator> = HashMap::new();

                    loop {
                        let chunk = {
                            let rx = chunk_rx.lock().unwrap();
                            rx.recv()
                        };
                        let Ok(chunk) = chunk else {
                            // Channel closed — no more work.
                            break;
                        };
                        Self::process_chunk(&chunk, &mut local);
                    }

                    // Send local results back. Ignore error (coordinator already gone).
                    let _ = result_tx.send(local);
                })
            })
            .collect();

        Pool {
            threads,
            chunk_rx,
            result_rx,
        }
    }

    /// Parse the byte slice line-by-line (separator `\n`) and update the local map.
    ///
    /// Lines are of the form `<station name>;<temperature>\n`.
    /// We parse temperatures as `f64` on-the-fly, but to avoid repeated
    /// heap allocation for station names the caller's map owns the string.
    fn process_chunk(chunk: &[u8], map: &mut HashMap<String, Aggregator>) {
        let mut start = 0;

        // When this chunk does *not* start at offset 0, the previous chunk
        // may have been cut mid-line.  We handle that at the chunking layer
        // by ensuring each chunk begins at a `\n` boundary (except the first).
        // Inside this function we just scan for `\n` separators.
        while start < chunk.len() {
            // Find the next newline.
            let end = match memchr(b'\n', &chunk[start..]) {
                Some(rel) => start + rel,
                None => {
                    // Incomplete last line – the caller should have made sure
                    // they only pass complete lines.  If this happens we skip.
                    break;
                }
            };

            let line = &chunk[start..end];
            start = end + 1;

            if line.is_empty() {
                continue;
            }

            // Split at ';'
            let Some(semi_pos) = memchr(b';', line) else {
                continue;
            };

            let station_bytes = &line[..semi_pos];
            let value_bytes = &line[semi_pos + 1..];
            let Ok(val) = fast_parse_f64(value_bytes) else {
                continue;
            };

            // We need an owned String for the HashMap key.
            // SAFETY: the input is valid UTF-8 from the measurements file.
            let station = unsafe { String::from_utf8_unchecked(station_bytes.to_vec()) };

            map.entry(station)
                .and_modify(|agg| agg.update(val))
                .or_insert_with(|| Aggregator::new(val));
        }
    }

    /// Wait for all workers to finish and collect their results.
    fn join(self) -> Vec<HashMap<String, Aggregator>> {
        // Drop the sender reference we hold so workers see closed channel.
        drop(self.chunk_rx);

        for handle in self.threads {
            handle.join().unwrap();
        }
        // Drain remaining results from the channel.
        let mut results = Vec::new();
        while let Ok(map) = self.result_rx.recv() {
            results.push(map);
        }
        results
    }
}

/// Parse a float in the 1BRC format (e.g. `23.5`, `-1.2`, `0.0`) from raw bytes.
///
/// Avoids std library overhead (locale handling, full decimal expansion).
/// Accepts: optional `-`, one or more digits, optionally `.` and one or more digits.
fn fast_parse_f64(bytes: &[u8]) -> Result<f64, ()> {
    if bytes.is_empty() {
        return Err(());
    }

    let mut i = 0;
    let negative = if bytes[i] == b'-' {
        i += 1;
        true
    } else {
        false
    };

    // Parse as (int_val * 10^frac_digits), then divide.
    let mut int_val: i64 = 0;
    let mut has_digits = false;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        int_val = int_val * 10 + (bytes[i] - b'0') as i64;
        i += 1;
        has_digits = true;
    }
    if !has_digits {
        return Err(());
    }

    let frac_digits;
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        frac_digits = (bytes.len() - i) as u32;
        while i < bytes.len() {
            int_val = int_val * 10 + (bytes[i] - b'0') as i64;
            i += 1;
        }
    } else {
        frac_digits = 0;
    }

    if i != bytes.len() {
        return Err(());
    }

    let divisor = 10i64.pow(frac_digits);
    let result = int_val as f64 / divisor as f64;
    if negative { Ok(-result) } else { Ok(result) }
}

fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Read the measurements file and split it into chunks, each aligned to
/// a newline boundary (except possibly the last chunk, which may not end
/// with a newline).
///
/// Each chunk is guaranteed to contain only complete lines.
fn chunk_file(path: &str, chunk_count: usize) -> io::Result<Vec<Vec<u8>>> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len() as usize;

    if chunk_count == 0 {
        return Ok(vec![]);
    }

    let chunk_size = (file_size + chunk_count - 1) / chunk_count;
    let mut chunks = Vec::with_capacity(chunk_count);

    let mut buffer = vec![0u8; file_size];
    file.read_exact(&mut buffer)?;

    let mut offset = 0;
    for i in 0..chunk_count {
        if offset >= file_size {
            break;
        }
        let end = if i == chunk_count - 1 {
            file_size
        } else {
            let mut end = (offset + chunk_size).min(file_size);
            // Align to the next newline so the chunk boundary doesn't cut a line.
            if end < file_size {
                // Find the next '\n' after `end`.
                while end < file_size && buffer[end] != b'\n' {
                    end += 1;
                }
                if end < file_size {
                    end += 1; // include the newline
                }
            }
            end
        };

        chunks.push(buffer[offset..end].to_vec());
        offset = end;
    }

    Ok(chunks)
}

/// Merge worker-local maps into a single sorted BTreeMap.
fn merge_results(
    results: Vec<HashMap<String, Aggregator>>,
) -> std::collections::BTreeMap<String, Aggregator> {
    let mut merged = std::collections::BTreeMap::new();
    for map in results {
        for (station, agg) in map {
            merged
                .entry(station)
                .and_modify(|existing: &mut Aggregator| existing.merge(&agg))
                .or_insert(agg);
        }
    }
    merged
}

/// Entry point called from the binary.
pub fn __main_by_ds_v4() {
    let chunks = chunk_file(crate::FILE, num_cpus()).unwrap();
    let (tx, rx) = mpsc::channel();
    for chunk in chunks {
        tx.send(chunk).unwrap();
    }

    let pool = Pool::new(num_cpus(), rx);
    let results = pool.join();
    let merged = merge_results(results);
    // Output in the required format.
    let mut output = String::from("{");
    for (i, (station, agg)) in merged.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        output.push_str(&format!(
            "{}={:.1}/{:.1}/{:.1}",
            station,
            agg.min(),
            agg.mean(),
            agg.max()
        ));
    }
    output.push('}');
    println!("{}", output);
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

pub fn __main() {
    let file = File::open(crate::FILE).unwrap();
    // 10M
    let reader = BufReader::with_capacity(10 * 1024 * 1024 * 1024, file);

    const CHUNK_SIZE: usize = 10000;

    let (tx, rx) = mpsc::channel::<HashMap<String, Aggregator>>();

    let mut handles = Vec::new();
    let mut chunk = Vec::with_capacity(CHUNK_SIZE);

    for line in reader.lines().map(|l| l.unwrap()) {
        chunk.push(line);
        if chunk.len() < CHUNK_SIZE {
            continue;
        }

        let tx = tx.clone();
        let lines = std::mem::take(&mut chunk);
        let handle = thread::spawn(move || {
            // process chunks
            let mut map: HashMap<String, Aggregator> = HashMap::new();
            for line in lines {
                let Some((station, val)) = line
                    .split_once(';')
                    .and_then(|(s, v)| v.parse::<f64>().ok().map(|val| (s, val)))
                else {
                    continue;
                };
                map.entry(station.to_string())
                    .and_modify(|agg| agg.update(val))
                    .or_insert_with(|| Aggregator::new(val));
            }
            tx.send(map).unwrap();
        });
        handles.push(handle);
    }

    if !chunk.is_empty() {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            let mut map: HashMap<String, Aggregator> = HashMap::new();
            for line in chunk {
                let Some((station, val)) = line
                    .split_once(';')
                    .and_then(|(s, v)| v.parse::<f64>().ok().map(|val| (s, val)))
                else {
                    continue;
                };
                map.entry(station.to_string())
                    .and_modify(|agg| agg.update(val))
                    .or_insert_with(|| Aggregator::new(val));
            }
            tx.send(map).unwrap();
        });
        handles.push(handle);
    }

    drop(tx);

    let mut reduce_map: HashMap<String, Aggregator> = HashMap::new();
    for map in rx {
        for (station, other) in map {
            reduce_map
                .entry(station)
                .and_modify(|agg| agg.merge(&other))
                .or_insert_with(|| other);
        }
    }

    // sort then print
    let mut out = reduce_map.into_iter().collect::<Vec<_>>();
    out.sort_by(|(a, _), (b, _)| a.cmp(b));
    print!("{{");
    for (i, (s, agg)) in out.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!("{}={:.1}/{:.1}/{:.1}", s, agg.min(), agg.mean(), agg.max());
    }
    println!("}}");

    for handle in handles {
        handle.join().unwrap();
    }
}
