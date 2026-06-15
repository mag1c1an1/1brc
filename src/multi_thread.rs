use std::{collections::HashMap, fs::File, io::Read, thread};

use crate::Aggregator;

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

pub fn __main() {
    let mut file = File::open(crate::FILE).unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    let mut bytes = Vec::with_capacity(file_size);
    file.read_to_end(&mut bytes).unwrap();

    let worker_count = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(bytes.len().max(1));
    let ranges = split_ranges(&bytes, worker_count);
    let file = File::open(crate::FILE).unwrap();
    // 8M
    let reader = BufReader::with_capacity(8 * 1024 * 1024 * 1024, file);

    const CHUNK_SIZE: usize = 5_000_000;

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
    for handle in handles {
        handle.join().unwrap();
    }
}
