use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    num::NonZero,
    sync::{Arc, Mutex, mpsc::Receiver},
    thread,
};

use memmap2::Mmap;

use crate::{Aggregator, FixedMap, output, process_line_and_compute_hash};

fn parse_temperature(bytes: &[u8]) -> Option<f64> {
    if bytes.is_empty() {
        return None;
    }

    let (negative, digits) = if bytes[0] == b'-' {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };

    let mut value = 0i32;
    let mut fraction_digits = 0u32;
    let mut after_decimal = false;
    let mut has_digit = false;

    for &byte in digits {
        match byte {
            b'0'..=b'9' => {
                value = value * 10 + i32::from(byte - b'0');
                has_digit = true;
                if after_decimal {
                    fraction_digits += 1;
                }
            }
            b'.' if !after_decimal => after_decimal = true,
            _ => return None,
        }
    }
    if !has_digit {
        return None;
    }

    let divisor = 10i32.pow(fraction_digits) as f64;
    let value = value as f64 / divisor;
    Some(if negative { -value } else { value })
}

fn process_range(bytes: &[u8]) -> HashMap<String, Aggregator> {
    let mut map: HashMap<String, Aggregator> = HashMap::with_capacity(512);
    let mut line_start = 0;

    while line_start < bytes.len() {
        let line_end = bytes[line_start..]
            .iter()
            .position(|&byte| byte == b'\n')
            .map_or(bytes.len(), |offset| line_start + offset);
        let line = &bytes[line_start..line_end];
        line_start = line_end.saturating_add(1);

        let Some(separator) = line.iter().position(|&byte| byte == b';') else {
            continue;
        };
        let station_bytes = &line[..separator];
        let Some(value) = parse_temperature(&line[separator + 1..]) else {
            continue;
        };

        // SAFETY: measurements.txt contains UTF-8 station names.
        let station = unsafe { std::str::from_utf8_unchecked(station_bytes) };
        if let Some(aggregator) = map.get_mut(station) {
            aggregator.update(value);
        } else {
            map.insert(station.to_owned(), Aggregator::new(value));
        }
    }

    map
}

fn split_ranges(bytes: &[u8], worker_count: usize) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::with_capacity(worker_count + 1);
    boundaries.push(0);

    for worker in 1..worker_count {
        let mut boundary = bytes.len() * worker / worker_count;
        while boundary < bytes.len() && bytes[boundary - 1] != b'\n' {
            boundary += 1;
        }
        boundaries.push(boundary);
    }
    boundaries.push(bytes.len());

    boundaries
        .windows(2)
        .filter_map(|range| (range[0] < range[1]).then_some((range[0], range[1])))
        .collect()
}

fn print_results(results: Vec<HashMap<String, Aggregator>>) {
    let mut merged: HashMap<String, Aggregator> = HashMap::with_capacity(512);
    for map in results {
        for (station, other) in map {
            merged
                .entry(station)
                .and_modify(|aggregator| aggregator.merge(&other))
                .or_insert(other);
        }
    }

    let mut stations = merged.into_iter().collect::<Vec<_>>();
    stations.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));

    print!("{{");
    for (index, (station, aggregator)) in stations.iter().enumerate() {
        if index > 0 {
            println!(",");
        }
        print!(
            "{}={:.1}/{:.1}/{:.1}",
            station,
            aggregator.min(),
            aggregator.mean(),
            aggregator.max()
        );
    }
    println!("}}");
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// A simple thread pool for parallelizing the 1BRC workload.
///
/// Workers pull chunks from a shared queue, process lines into
/// per-station `Aggregator` maps, and send results back to the
/// coordinator thread via a channel.
struct Pool {
    threads: Vec<std::thread::JoinHandle<()>>,
    /// Shared iterator that yields raw byte chunks for each worker.
    chunk_rx: Arc<Mutex<Receiver<(Arc<Mmap>, usize, usize)>>>,
    /// Channel for workers to send their local aggregation results back.
    result_rx: Receiver<HashMap<String, Aggregator>>,
}

impl Pool {
    /// Spawn `worker_count` threads. Each worker repeatedly:
    ///  1. Locks the shared receiver and pops a chunk.
    ///  2. Parses the chunk into station→Aggregator entries.
    ///  3. Sends the local map back through `result_tx`.
    fn new(worker_count: usize, chunk_rx: Receiver<(Arc<Mmap>, usize, usize)>) -> Self {
        let chunk_rx = Arc::new(Mutex::new(chunk_rx));
        let (result_tx, result_rx) = std::sync::mpsc::channel();

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
                        let Ok((mmap, start, end)) = chunk else {
                            // Channel closed — no more work.
                            break;
                        };
                        Self::process_chunk(&mmap[start..end], &mut local);
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
fn chunk_file(path: &str, chunk_count: usize) -> std::io::Result<(Mmap, Vec<(usize, usize)>)> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len() as usize;

    if chunk_count == 0 {
        return Ok((unsafe { Mmap::map(&file)? }, vec![]));
    }

    // SAFETY: the measurements file is read-only and we only create shared references.
    let mmap = unsafe { Mmap::map(&file)? };

    let chunk_size = (file_size + chunk_count - 1) / chunk_count;
    let mut ranges = Vec::with_capacity(chunk_count);

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
                while end < file_size && mmap[end] != b'\n' {
                    end += 1;
                }
                if end < file_size {
                    end += 1; // include the newline
                }
            }
            end
        };

        ranges.push((offset, end));
        offset = end;
    }

    Ok((mmap, ranges))
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

// Entry point called from the binary.
pub fn deepseek() {
    let (mmap, ranges) = chunk_file(crate::FILE, num_cpus()).unwrap();
    let mmap = Arc::new(mmap);
    let (tx, rx) = std::sync::mpsc::channel();
    for (start, end) in ranges {
        tx.send((Arc::clone(&mmap), start, end)).unwrap();
    }
    drop(tx);

    let pool = Pool::new(num_cpus(), rx);
    let results = pool.join();
    let merged = merge_results(results);
    print!("{{");
    for (i, (buf, agg)) in merged.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!(
            "{}={:.1}/{:.1}/{:.1}",
            buf,
            agg.min(),
            agg.mean(),
            agg.max()
        );
    }
    println!("}}");
}

fn codex() {
    let mut file = File::open(crate::FILE).unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    let mut bytes = Vec::with_capacity(file_size);
    file.read_to_end(&mut bytes).unwrap();

    let worker_count = std::thread::available_parallelism()
        .map(|c| c.get())
        .unwrap_or(1)
        .min(bytes.len().max(1));

    let ranges = split_ranges(&bytes, worker_count);

    let results = thread::scope(|scope| {
        let bytes = &bytes;
        ranges
            .into_iter()
            .map(|(start, end)| scope.spawn(move || process_range(&bytes[start..end])))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect()
    });
    print_results(results);
}

fn v1() {
    let file = File::open(crate::FILE).unwrap();
    // 8M
    let reader = BufReader::with_capacity(8 * 1024 * 1024 * 1024, file);

    const CHUNK_SIZE: usize = 5_000_000;

    let (tx, rx) = std::sync::mpsc::channel::<HashMap<String, Aggregator>>();

    let mut handles = Vec::new();

    let reduce = thread::spawn(move || {
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
    });

    handles.push(reduce);

    let mut chunk = Vec::with_capacity(CHUNK_SIZE);

    for line in reader.lines().map(|l| l.unwrap()) {
        chunk.push(line);
        if chunk.len() < CHUNK_SIZE {
            continue;
        }

        let tx = tx.clone();
        let lines = std::mem::replace(&mut chunk, Vec::with_capacity(CHUNK_SIZE));
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

    for handle in handles {
        handle.join().unwrap();
    }
}

#[inline(always)]
fn next_newline(data: &[u8], mut i: usize) -> usize {
    while i < data.len() && data[i] != b'\n' {
        i += 1;
    }
    i
}

fn process_chunk(chunk: &[u8], map: &mut FixedMap) {
    let mut start = 0;

    for i in 0..chunk.len() {
        // todo(jiax): use memchr
        if chunk[i] == b'\n' {
            if i > start {
                process_line_and_compute_hash(&chunk[start..i], map);
            }
            start = i + 1;
        }
    }

    // 处理最后一行没有 '\n' 的情况
    if start < chunk.len() {
        process_line_and_compute_hash(&chunk[start..], map);
    }
}

fn v2() {
    // mmap
    let file = File::open(crate::FILE).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let n = num_cpus();
    let len = mmap.len();
    std::thread::scope(|s| {
        let (tx, rx) = std::sync::mpsc::channel::<FixedMap>();
        for tid in 0..n {
            let data = &mmap;
            let tx = tx.clone();
            s.spawn(move || {
                let raw_start = len * tid / n;
                let raw_end = len * (tid + 1) / n;
                let start = if tid == 0 {
                    0
                } else {
                    next_newline(data, raw_start) + 1
                };
                let end = if tid + 1 == n {
                    len
                } else {
                    next_newline(data, raw_end) + 1
                };
                let mut map = FixedMap::with_capacity(1 << 14);
                process_chunk(&data[start..end], &mut map);
                tx.send(map).unwrap();
            });
        }
        drop(tx);

        let mut reduce = FixedMap::with_capacity(1 << 14);
        for other in rx {
            reduce.merge(other);
        }
        let entries = reduce.finish();
        output(entries);
    });
}

pub fn __main() {
    v2();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_temperatures() {
        assert_eq!(parse_temperature(b"23.5"), Some(23.5));
        assert_eq!(parse_temperature(b"-1.2"), Some(-1.2));
        assert_eq!(parse_temperature(b"0.0"), Some(0.0));
        assert_eq!(parse_temperature(b""), None);
        assert_eq!(parse_temperature(b"-"), None);
    }

    #[test]
    fn processes_a_range_without_a_trailing_newline() {
        let map = process_range(b"Alpha;1.0\nBeta;-2.5\nAlpha;3.0");

        assert_eq!(map["Alpha"].min(), 1.0);
        assert_eq!(map["Alpha"].mean(), 2.0);
        assert_eq!(map["Alpha"].max(), 3.0);
        assert_eq!(map["Beta"].mean(), -2.5);
    }

    #[test]
    fn split_ranges_preserves_complete_lines() {
        let bytes = b"A;1.0\nLong station;2.0\nC;3.0\n";
        let ranges = split_ranges(bytes, 3);
        let joined = ranges
            .iter()
            .flat_map(|&(start, end)| &bytes[start..end])
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(joined, bytes);
        assert!(ranges.iter().all(|&(start, end)| {
            start == 0
                || bytes[start - 1] == b'\n' && (end == bytes.len() || bytes[end - 1] == b'\n')
        }));
    }
}
