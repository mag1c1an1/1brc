// Rust translation of CreateMeasurements3.java from the 1BRC project.
//
// Expects: data/weather_stations.csv
// Writes:  measurements3.txt

use rand::RngExt;
use rand_distr::{Distribution, Normal};
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    process,
    time::Instant,
};

const MAX_NAME_LEN: usize = 100;
const KEYSET_SIZE: usize = 10_000;

#[derive(Debug, Clone)]
struct WeatherStation {
    name: String,
    avg_temp: f32,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: create_measurements3 <number of records to create>");
        process::exit(1);
    }

    let size: usize = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid value for <number of records to create>");
            eprintln!("Usage: create_measurements3 <number of records to create>");
            process::exit(1);
        }
    };

    let weather_stations = generate_weather_stations()?;
    let start = Instant::now();
    let mut rng = rand::rng();

    let file = File::create("measurements3.txt")?;
    let mut out = BufWriter::with_capacity(16 * 1024 * 1024, file);

    for i in 1..=size {
        let station = &weather_stations[rng.random_range(0..weather_stations.len())];
        let normal = Normal::new(station.avg_temp as f64, 7.0).unwrap();
        let temp = (normal.sample(&mut rng) * 10.0).round() / 10.0;

        writeln!(out, "{};{:.1}", station.name, temp)?;

        if i % 50_000_000 == 0 {
            println!(
                "Wrote {} measurements in {} ms",
                format_number(i),
                start.elapsed().as_millis()
            );
        }
    }

    out.flush()?;
    Ok(())
}

fn generate_weather_stations() -> io::Result<Vec<WeatherStation>> {
    // Use the public list of city names and concatenate them into a long string,
    // used as a source of station-name randomness.
    let rows = read_data_rows("data/weather_stations.csv")?;

    let mut big_name = String::with_capacity(1 << 20);
    for row in &rows {
        if let Some((city, _lat)) = row.split_once(';') {
            big_name.push_str(city);
        }
    }

    let mut weather_stations = Vec::with_capacity(KEYSET_SIZE);
    let mut names = HashSet::with_capacity(KEYSET_SIZE);
    let mut min_len = usize::MAX;
    let mut max_len = 0usize;
    let mut name_source = NameSource::new(big_name);
    let mut rng = rand::rng();

    let y_offset = 4.0_f64;
    let factor = 2500.0_f64;
    let x_offset = 0.372_f64;
    let power = 7.0_f64;

    for row in rows.iter().take(KEYSET_SIZE) {
        let Some((_city, lat_str)) = row.split_once(';') else {
            continue;
        };

        // Use a 7th-order curve to simulate the station-name length distribution:
        // mostly short names, with some large outliers.
        let raw_len = y_offset + factor * (rng.random::<f64>() - x_offset).powf(power);
        let name_len = raw_len as usize;
        let name_len = name_len.clamp(1, MAX_NAME_LEN);

        let mut name_buf = name_source.read_chars(name_len)?;

        if name_buf.first().is_some_and(|ch| ch.is_whitespace()) {
            name_buf[0] = name_source.read_non_space()?;
        }
        if name_buf.last().is_some_and(|ch| ch.is_whitespace()) {
            let last = name_buf.len() - 1;
            name_buf[last] = name_source.read_non_space()?;
        }

        let mut name: String = name_buf.iter().collect();
        while names.contains(&name) {
            let idx = rng.random_range(0..name_buf.len());
            name_buf[idx] = name_source.read_non_space()?;
            name = name_buf.iter().collect();
        }

        while name.len() > 100 {
            name_buf.pop();
            if name_buf.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "generated empty station name",
                ));
            }
            if name_buf.last().is_some_and(|ch| ch.is_whitespace()) {
                let last = name_buf.len() - 1;
                name_buf[last] = name_source.read_non_space()?;
            }
            name = name_buf.iter().collect();
        }

        if name.contains(';') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "station name contains a semicolon",
            ));
        }

        let actual_len = name.len(); // UTF-8 byte length, same role as Java getBytes(UTF_8).length
        names.insert(name.clone());
        min_len = min_len.min(actual_len);
        max_len = max_len.max(actual_len);

        let lat: f32 = lat_str.trim().parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid latitude in row {row:?}: {e}"),
            )
        })?;

        // Guesstimate mean temperature using cosine of latitude.
        let avg_temp = (30.0_f32 * lat.to_radians().cos()) - 10.0;
        weather_stations.push(WeatherStation { name, avg_temp });
    }

    println!(
        "Generated {} station names with length from {} to {}",
        format_number(KEYSET_SIZE),
        format_number(min_len),
        format_number(max_len)
    );

    Ok(weather_stations)
}

fn read_data_rows(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        rows.push(line);
    }

    Ok(rows)
}

struct NameSource {
    chars: Vec<char>,
    pos: usize,
}

impl NameSource {
    fn new(s: String) -> Self {
        Self {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn read_chars(&mut self, count: usize) -> io::Result<Vec<char>> {
        if self.pos + count > self.chars.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "name source exhausted",
            ));
        }
        let out = self.chars[self.pos..self.pos + count].to_vec();
        self.pos += count;
        Ok(out)
    }

    fn read_non_space(&mut self) -> io::Result<char> {
        loop {
            if self.pos >= self.chars.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "name source exhausted",
                ));
            }
            let ch = self.chars[self.pos];
            self.pos += 1;
            if ch != ' ' {
                return Ok(ch);
            }
        }
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    let first_group_len = s.len() % 3;

    if first_group_len != 0 {
        out.push_str(&s[..first_group_len]);
    }

    for chunk_start in (first_group_len..s.len()).step_by(3) {
        if !out.is_empty() {
            out.push(',');
        }
        out.push_str(&s[chunk_start..chunk_start + 3]);
    }

    out
}
