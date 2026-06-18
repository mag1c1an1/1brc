# key note

100,000,000

single thread read

```text
Benchmark 1: target/release/maji > /dev/null
  Time (mean ± σ):     118.5 ms ±   1.6 ms    [User: 9.1 ms, System: 108.6 ms]
  Range (min … max):   115.9 ms … 122.2 ms    25 runs
```


# baseline

100,000,000

hyperfine --warmup 3 'cargo run --release --bin baseline > /dev/null'
Benchmark 1: cargo run --release --bin baseline > /dev/null
  Time (mean ± σ):      9.662 s ±  0.068 s    [User: 9.540 s, System: 0.113 s]
  Range (min … max):    9.507 s …  9.768 s    10 runs

---
multithread


perf 降低频率
