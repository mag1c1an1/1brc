use std::f64;

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
