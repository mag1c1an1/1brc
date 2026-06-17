#![allow(unused)]

use std::{
    arch::is_aarch64_feature_detected, collections::hash_map::Keys, f64, hash::Hasher, panic,
};

use fxhash::{FxHasher, hash64};

pub mod async_io;
pub mod multi_thread;
pub mod single_thread;

pub static FILE: &str = "./measurements.txt";

#[derive(Debug, Clone)]
pub struct Aggregator {
    min: f64,
    max: f64,
    sum: f64,
    cnt: i64,
}

impl Aggregator {
    pub fn new(val: f64) -> Self {
        Self {
            min: val,
            max: val,
            sum: val,
            cnt: 1,
        }
    }

    pub fn update(&mut self, val: f64) {
        self.min = self.min.min(val);
        self.max = self.max.max(val);
        self.sum += val;
        self.cnt += 1;
    }

    pub fn mean(&self) -> f64 {
        self.sum / self.cnt as f64
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    /// Merge another Aggregator into this one (combining min/max/sum/cnt).
    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.cnt += other.cnt;
    }
}

#[derive(Clone)]
pub struct I64Aggregator {
    min: i64,
    max: i64,
    sum: i64,
    cnt: i64,
}

impl I64Aggregator {
    pub fn new(val: i64) -> Self {
        Self {
            min: val,
            max: val,
            sum: val,
            cnt: 1,
        }
    }

    pub fn update(&mut self, val: i64) {
        self.min = self.min.min(val);
        self.max = self.max.max(val);
        self.sum += val;
        self.cnt += 1;
    }

    pub fn mean(&self) -> f64 {
        self.sum as f64 / self.cnt as f64 / 10.0
    }

    pub fn min(&self) -> f64 {
        self.min as f64 / 10.0
    }

    pub fn max(&self) -> f64 {
        self.max as f64 / 10.0
    }

    /// Merge another Aggregator into this one (combining min/max/sum/cnt).
    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.cnt += other.cnt;
    }
}

impl Default for I64Aggregator {
    fn default() -> Self {
        Self::new(0)
    }
}

pub fn parse_temperature_by_bytes(val: &[u8]) -> i64 {
    let mut ans = 0;
    let mut i = 0;
    let is_negative = val[0] == b'-';
    if is_negative {
        i += 1;
    }
    while val[i] != b'.' {
        ans = ans * 10 + (val[i] - b'0') as i64;
        i += 1;
    }
    ans = ans * 10 + (val[i + 1] - b'0') as i64;
    if is_negative { -ans } else { ans }
}

/// parse a temperature string into an i64 (e.g. "12.3" -> 123)
pub fn parse_temperature(val: &str) -> i64 {
    let mut is_negative = false;
    let mut ans = 0;

    for (i, c) in val.chars().enumerate() {
        match c {
            '-' => {
                if i != 0 {
                    panic!("wrong: {val}");
                }
                is_negative = true;
            }
            '.' => {
                continue;
            }
            '0'..='9' => ans = ans * 10 + (c as u8 - b'0') as i64,
            other => panic!("wrong: {other}"),
        }
    }
    if is_negative { -ans } else { ans }
}

#[derive(Default, Clone)]
struct Entry {
    hash: u64,
    key: [u8; 32],
    len: u8,
    agg: I64Aggregator,
    used: bool,
}

struct FixedMap {
    cap: usize,
    entries: Vec<Entry>,
}

impl FixedMap {
    fn with_capacity(cap: usize) -> Self {
        if !cap.is_power_of_two() {
            panic!("cap must be a power of two: {cap}")
        }
        Self {
            cap,
            entries: vec![Entry::default(); cap],
        }
    }

    fn update(&mut self, key: &[u8], value: i64) {
        let hash = fast_hash(key);
        self.update_with_hash(hash, key, value);
    }

    fn update_with_hash(&mut self, hash: u64, key: &[u8], value: i64) {
        let mut idx = (hash as usize) & (self.cap - 1);
        loop {
            let e = &mut self.entries[idx];
            if !e.used {
                e.used = true;
                e.hash = hash;
                e.key[..key.len()].copy_from_slice(key);
                e.len = key.len() as u8;
                e.agg = I64Aggregator::new(value);
                return;
            }

            if e.hash == hash && e.len as usize == key.len() && &e.key[..e.len as usize] == key {
                e.agg.update(value);
                return;
            }

            idx = (idx + 1) & (self.cap - 1);
        }
    }

    pub fn finish(self) -> Vec<(Vec<u8>, I64Aggregator)> {
        let mut result = self
            .entries
            .into_iter()
            .filter(|e| e.used)
            .map(|e| (e.key[..e.len as usize].to_vec(), e.agg))
            .collect::<Vec<_>>();

        // result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

// // FNV-1a hash function
// fn fast_hash(key: &[u8]) -> u64 {
//     let mut hash = 0xcbf29ce484222325u64;
//     for &b in key {
//         hash ^= b as u64;
//         hash = hash.wrapping_mul(0x100000001b3);
//     }
//     hash
// }

// use fxhash
fn fast_hash(key: &[u8]) -> u64 {
    let mut h = FxHasher::default();
    h.write(key);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_temperature() {
        assert_eq!(parse_temperature("11.1"), 111);
        assert_eq!(parse_temperature("0"), 0);
        assert_eq!(parse_temperature("0.0"), 0);
        assert_eq!(parse_temperature("1.1"), 11);
        assert_eq!(parse_temperature("-11.1"), -111);
        assert_eq!(parse_temperature("-0.1"), -1);
        assert_eq!(parse_temperature("-0.0"), 0);
    }
    #[test]
    fn test_parse_temperature_by_bytes() {
        assert_eq!(parse_temperature_by_bytes(b"11.1"), 111);
        assert_eq!(parse_temperature_by_bytes(b"0.0"), 0);
        assert_eq!(parse_temperature_by_bytes(b"1.1"), 11);
        assert_eq!(parse_temperature_by_bytes(b"-11.1"), -111);
        assert_eq!(parse_temperature_by_bytes(b"-0.1"), -1);
        assert_eq!(parse_temperature_by_bytes(b"-0.0"), 0);
    }
}
