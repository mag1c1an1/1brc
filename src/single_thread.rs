use std::{
    collections::HashMap,
    fs::File,
    hash::Hasher,
    io::{BufRead, BufReader},
};

use fxhash::{FxBuildHasher, FxHashMap, FxHasher};

use crate::{
    Aggregator, FixedMap, I64Aggregator, parse_temperature, parse_temperature_by_bytes,
    parse_temperature_by_trick, process_line_and_compute_hash,
};

pub fn v1() {
    let file = File::open(crate::FILE).unwrap();
    let reader = BufReader::new(file);
    let mut map: HashMap<String, Aggregator> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap(); // String，不包含末尾换行符
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

    let mut entries = map.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(s1, _), (s2, _)| s1.cmp(s2));

    print!("{{");
    for (i, (s, agg)) in entries.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!("{}={:.1}/{:.1}/{:.1}", s, agg.min(), agg.mean(), agg.max());
    }
    println!("}}");
}

pub fn v2() {
    let file = File::open(crate::FILE).unwrap();
    let mut reader = BufReader::new(file);
    let mut map: HashMap<String, I64Aggregator> = HashMap::with_capacity(512); // 1brc only has 413 stations

    let mut buf = String::new();

    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Err(e) => {
                panic!("{}", e)
            }
            _ => {
                let line = buf.trim();
                let Some((station, val)) =
                    line.split_once(';').map(|(s, v)| (s, parse_temperature(v)))
                else {
                    continue;
                };
                if let Some(agg) = map.get_mut(station) {
                    agg.update(val);
                } else {
                    map.insert(station.to_string(), I64Aggregator::new(val));
                }
                buf.clear();
            }
        }
    }

    let mut entries = map.into_iter().collect::<Vec<_>>();
    entries.sort_by(|(s1, _), (s2, _)| s1.cmp(s2));

    print!("{{");
    for (i, (s, agg)) in entries.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!("{}={:.1}/{:.1}/{:.1}", s, agg.min(), agg.mean(), agg.max());
    }
    println!("}}");
}

fn read_file_by_read_line(
    reader: &mut BufReader<File>,
    map: &mut FxHashMap<String, I64Aggregator>,
    buf: &mut String,
) {
    loop {
        match reader.read_line(buf) {
            Ok(0) => break,
            Err(e) => {
                panic!("{}", e)
            }
            _ => {
                let line = buf.trim();
                let Some((station, val)) =
                    line.split_once(';').map(|(s, v)| (s, parse_temperature(v)))
                else {
                    continue;
                };
                map.entry(station.to_string())
                    .and_modify(|agg| agg.update(val))
                    .or_insert_with(|| I64Aggregator::new(val));
                buf.clear();
            }
        }
    }
}

fn process_line(line: &[u8], map: &mut FixedMap) {
    process_line_and_compute_hash(line, map);
    // process_line_inner(line, map);
}

fn process_line_inner(line: &[u8], map: &mut FixedMap) {
    let mut semi = 0;
    while line[semi] != b';' {
        semi += 1;
    }
    let station = &line[..semi];
    let temp = &line[semi + 1..];
    let value = parse_temperature_by_trick(temp);

    map.update(station, value);
}



// parse
fn read_file_by_bytes(reader: &mut BufReader<File>, map: &mut FixedMap) {
    let mut leftover = Vec::new();
    loop {
        let content = reader.fill_buf().unwrap();
        if content.is_empty() {
            break;
        }
        let mut start = 0;
        for i in 0..content.len() {
            if content[i] == b'\n' {
                if leftover.is_empty() {
                    process_line(&content[start..i], map);
                } else {
                    leftover.extend_from_slice(&content[start..i]);
                    process_line(&leftover, map);
                    leftover.clear();
                }
                start = i + 1;
            }
        }
        if start < content.len() {
            leftover.extend_from_slice(&content[start..]);
        }
        let consumued = content.len(); // borrow checker here is stupid
        reader.consume(consumued);
    }
    if !leftover.is_empty() {
        process_line(&leftover, map);
    }
}

pub fn v3() {
    let file = File::open(crate::FILE).unwrap();
    let mut reader = BufReader::with_capacity(8 * (1 << 10), file);
    // let mut map: FxHashMap<Vec<u8>, I64Aggregator> =
    //     FxHashMap::with_capacity_and_hasher(512, FxBuildHasher::new()); // 1brc only has 413 stations
    let mut map = FixedMap::with_capacity(16384);

    read_file_by_bytes(&mut reader, &mut map);

    // loop {
    //     match reader.read_line(&mut buf) {
    //         Ok(0) => break,
    //         Err(e) => {
    //             panic!("{}", e)
    //         }
    //         _ => {
    //             let line = buf.trim();
    //             let Some((station, val)) =
    //                 line.split_once(';').map(|(s, v)| (s, parse_temperature(v)))
    //             else {
    //                 continue;
    //             };
    //             map.entry(station.to_string())
    //                 .and_modify(|agg| agg.update(val))
    //                 .or_insert_with(|| I64Aggregator::new(val));
    //             buf.clear();
    //         }
    //     }
    // }

    // let mut entries = map.into_iter().collect::<Vec<_>>();
    // let mut entries = map.finish();
    // entries.sort_by(|(s1, _), (s2, _)| s1.cmp(s2));
    let entries = map.finish();

    print!("{{");
    for (i, (buf, agg)) in entries.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!(
            "{}={:.1}/{:.1}/{:.1}",
            String::from_utf8_lossy(buf),
            // buf,
            agg.min(),
            agg.mean(),
            agg.max()
        );
    }
    println!("}}");
}

pub fn __main() {
    v3()
}
