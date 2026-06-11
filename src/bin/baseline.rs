use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead, BufReader},
};

use onebrc::Aggregator;

fn main() -> io::Result<()> {
    let file = File::open(onebrc::FILE)?;
    let reader = BufReader::new(file);
    let mut map: BTreeMap<String, Aggregator> = BTreeMap::new();

    for line in reader.lines() {
        let line = line?; // String，不包含末尾换行符
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
    print!("{{");
    for (i, (s, agg)) in map.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!("{}={:.1}/{:.1}/{:.1}", s, agg.min(), agg.mean(), agg.max());
    }
    println!("}}");
    Ok(())
}
