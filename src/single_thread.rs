use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{Aggregator, I64Aggregator, parse_temperature};

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

pub fn __main() {
    v2()
}
