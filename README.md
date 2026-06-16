# baseline

100,000,000

hyperfine --warmup 3 'cargo run --release --bin baseline > /dev/null'
Benchmark 1: cargo run --release --bin baseline > /dev/null
  Time (mean ± σ):      9.662 s ±  0.068 s    [User: 9.540 s, System: 0.113 s]
  Range (min … max):    9.507 s …  9.768 s    10 runs

---
multithread
