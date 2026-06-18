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

--- 1,000,000,000

baseline: 
```text
hyperfine --warmup 3 'target/release/baseline > /dev/null'
Benchmark 1: target/release/baseline > /dev/null
  Time (mean ± σ):     93.786 s ±  2.240 s    [User: 90.985 s, System: 1.343 s]
  Range (min … max):   90.903 s … 98.151 s    10 runs
```


deepseek:
```text
hyperfine --warmup 3 'target/release/maji > /dev/null'
Benchmark 1: target/release/maji > /dev/null
  Time (mean ± σ):      2.657 s ±  0.143 s    [User: 39.207 s, System: 0.408 s]
  Range (min … max):    2.480 s …  2.910 s    10 runs
```

codex:
```text
hyperfine --warmup 3 'target/release/maji > /dev/null'
Benchmark 1: target/release/maji > /dev/null
  Time (mean ± σ):      7.744 s ±  0.141 s    [User: 34.126 s, System: 4.647 s]
  Range (min … max):    7.585 s …  8.048 s    10 runs
```
