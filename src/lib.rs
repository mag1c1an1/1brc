use std::{f64, panic};

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
}
