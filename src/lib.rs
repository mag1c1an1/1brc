use std::f64;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

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
}
