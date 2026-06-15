# 失败的多线程v1
```rust
pub fn main() {
    let file = File::open(crate::FILE).unwrap();
    // 10M
    let reader = BufReader::with_capacity(10 * 1024 * 1024 * 1024, file);

    const CHUNK_SIZE: usize = 10000;

    let (tx, rx) = mpsc::channel::<HashMap<String, Aggregator>>();

    let mut handles = Vec::new();
    let mut chunk = Vec::with_capacity(CHUNK_SIZE);

    for line in reader.lines().map(|l| l.unwrap()) {
        chunk.push(line);
        if chunk.len() < CHUNK_SIZE {
            continue;
        }

        let tx = tx.clone();
        let lines = std::mem::take(&mut chunk);
        let handle = thread::spawn(move || {
            // process chunks
            let mut map: HashMap<String, Aggregator> = HashMap::new();
            for line in lines {
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
            tx.send(map).unwrap();
        });
        handles.push(handle);
    }

    if !chunk.is_empty() {
        let tx = tx.clone();
        let handle = thread::spawn(move || {
            let mut map: HashMap<String, Aggregator> = HashMap::new();
            for line in chunk {
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
            tx.send(map).unwrap();
        });
        handles.push(handle);
    }

    drop(tx);

    let mut reduce_map: HashMap<String, Aggregator> = HashMap::new();
    for map in rx {
        for (station, other) in map {
            reduce_map
                .entry(station)
                .and_modify(|agg| agg.merge(&other))
                .or_insert_with(|| other);
        }
    }

    // sort then print
    let mut out = reduce_map.into_iter().collect::<Vec<_>>();
    out.sort_by(|(a, _), (b, _)| a.cmp(b));
    print!("{{");
    for (i, (s, agg)) in out.iter().enumerate() {
        if i > 0 {
            println!(",");
        }
        print!("{}={:.1}/{:.1}/{:.1}", s, agg.min(), agg.mean(), agg.max());
    }
    println!("}}");

    for handle in handles {
        handle.join().unwrap();
    }
}
```

real	0m16.319s
user	0m30.953s
sys	0m1.125s

---
codex:

```plaintext
• 我会先读 maji.rs、baseline.rs 和 justfile，确认基准命令、输入规模和两条实现路
  径；然后实际跑测试并用构建配置、系统时间和内存数据定位差异。

• Explored
  └ Read maji.rs, justfile, Cargo.toml, baseline.rs

• Ran git status --short
  └ M  .cargo/config.toml
    M  .gitignore
    … +8 lines (ctrl + t to view transcript)
    ?? baseline.ans
    ?? maji.ans

────────────────────────────────────────────────────────────────────────────────

• 代码路径已经很明确：maji 不是“多线程版 baseline”，而是 10GB 读缓冲、每 1 万行
  创建一个 OS 线程，并且读完整个文件后才消费结果 channel。先确认输入文件规模和机
  器并行度，再按 justfile 的 release 配置测 wall time 与峰值 RSS。

• Ran nproc
  └ 20

• Explored
  └ List measurements.txt
    Read multi_thread.rs

• Ran command -v just; command -v /usr/bin/time; command -v perf; command -v
  │ cargo-flamegraph
  └ /etc/profiles/per-user/maji/bin/just
    /run/current-system/sw/bin/perf

• Ran free -h
  └                total        used        free      shared  buff/cache
    available
    Mem:            30Gi       5.0Gi       9.0Gi       108Mi        17Gi
    25Gi
    Swap:             0B          0B          0B

────────────────────────────────────────────────────────────────────────────────

• 输入是 1.3GB、机器有 20 个逻辑 CPU。这个规模下会创建大约 1 万个线程和 1 万份局
  部 HashMap；而 baseline 始终只有一份聚合表，所以从复杂度上 maji 已经天然承担了
  大量额外分配、线程调度和二次归并。现在先重建 release，再分别测两者。

• Ran type -a time
  └ time is a shell keyword
    time is /run/current-system/sw/bin/time

• Ran cargo build --release --bin baseline --bin maji
  └     Finished `release` profile [optimized + debuginfo] target(s) in 0.02s

• Explored
  └ Search ^ in measurements.txt

• Ran find /nix/store -path '*/bin/time' -type f 2>/dev/null | head -5
  └ /nix/store/yczmlfdicv5vq2z3aigb29lkp9bx0llz-time-1.9/bin/time
    /nix/store/n0wrh3vjfwcqfyswwai0zcxvkpibq34v-time-1.10/bin/time

────────────────────────────────────────────────────────────────────────────────

• 文件确认为 1 亿行。前面的行数统计已经把文件页缓存预热，因此接下来的比较主要反
  映解析、分配和调度成本，而不是第一次磁盘读取的偶然差异；我会顺序执行，避免两个
  程序争用 CPU 和内存带宽。

• Ran /run/current-system/sw/bin/time -v target/release/baseline
  └ {Abha=-27.0/18.0/63.6,
    Abidjan=-19.9/26.0/72.4,
    … +432 lines (ctrl + t to view transcript)
        Page size (bytes): 4096
        Exit status: 0

────────────────────────────────────────────────────────────────────────────────

• baseline 的实测是 9.74s、峰值 RSS 约 2.1MiB，CPU 基本单核满载。现在测 maji；重
  点观察 system time、上下文切换和峰值 RSS，它们能直接验证线程与分配开销。

• Ran /run/current-system/sw/bin/time -v target/release/maji > /dev/null
  └     Command being timed: "target/release/maji"
        User time (seconds): 31.80
    … +19 lines (ctrl + t to view transcript)
        Page size (bytes): 4096
        Exit status: 0

────────────────────────────────────────────────────────────────────────────────

• 结果验证了主要瓶颈：maji 用了 17.05s，但累计 CPU 时间约 33s，平均只利用 1.93
  个核；峰值 RSS 是 1,849,652KiB，约 1.76GiB。也就是说它付出了多线程成本，却没有
  形成持续并行。下一步我会用计数器和输出比对区分解析成本、线程调度与实现正确性。

• Ran wc -c baseline.ans maji.ans
  └ 10664 baseline.ans
    10664 maji.ans
    21328 total

• Ran cmp -s baseline.ans maji.ans; printf '%s\n' $?
  └ 0

• Ran command -v strace; rustc --version; cargo --version
  └ /run/current-system/sw/bin/strace
    rustc 1.98.0-nightly (3daae5e42 2026-06-14)
    cargo 1.98.0-nightly (fe63976b2 2026-06-11)

• Ran perf stat -e
  │ task-clock,context-switches,cpu-migrations,page-faults,cycles,instructions
  │ ,branches,branch-misses target/release/baseline > /dev/null
  └
     Performance counter stats for 'target/release/baseline':
    … +18 lines (ctrl + t to view transcript)



• Ran perf stat -e
  │ task-clock,context-switches,cpu-migrations,page-faults,cycles,instructions
  │ ,branches,branch-misses target/release/maji > /dev/null
  └
     Performance counter stats for 'target/release/maji':
    … +18 lines (ctrl + t to view transcript)


• 输出完全一致，所以性能差异不是结果错误造成的。硬件计数显示 maji 约消耗 33.8s
  task-clock，而 baseline 只有 9.8s，总 CPU 工作量是约 3.4 倍；我再统计关键系统
  调用，确认 10GB 缓冲实际如何读文件，以及线程创建次数。

✔ You approved codex to run strace -f -c -e
  trace=read,clone,clone3,mmap,munmap,madvise target/release/ba... this time

• Ran strace -f -c -e trace=read,clone,clone3,mmap,munmap,madvise target/
  │ release/baseline > /dev/null
  └ % time     seconds  usecs/call     calls    errors syscall
    ------ ----------- ----------- --------- --------- ----------------
    … +3 lines (ctrl + t to view transcript)
    ------ ----------- ----------- --------- --------- ----------------
    100.00    0.013016           0    168425           total

✔ You approved codex to run strace -f -c -e
  trace=read,clone,clone3,mmap,munmap,madvise target/release/ma... this time

• Ran strace -f -c -e trace=read,clone,clone3,mmap,munmap,madvise target/
  │ release/maji > /dev/null
  └ strace: Process 161609 attached
    strace: Process 161610 attached
    … +10005 lines (ctrl + t to view transcript)
    ------ ----------- ----------- --------- --------- ----------------
    100.00    0.537925           7     70035           total

────────────────────────────────────────────────────────────────────────────────

• 系统调用统计把结构性问题钉死了：baseline 有 168,412 次小块 read，但没有线程创
  建；maji 只做了 9 次 read，却调用了恰好 10,000 次 clone3，同时产生约 20,000 次
  线程栈 mmap/munmap。大读缓冲减少了 syscall，却把整个 1.3GB 文件驻留在进程内存
  中；省下的 syscall 时间远小于内存和线程代价。

• Ran stat -c '%s bytes' measurements.txt
  └ 1379572761 bytes

• Ran wc -l baseline.ans
  └ 413 baseline.ans

• Explored
  └ Read baseline.rs, maji.rs, justfile

• Ran git diff --check
  └ (no output)
```

**实测结果**

按 `justfile` 对应的 release 构建测试，输入为 1 亿行、1.38GB：

| 指标 | baseline | maji |
|---|---:|---:|
| Wall time | 9.74s | 17.05s |
| CPU time | 9.84s | 32.97s |
| 平均 CPU | 0.99 核 | 1.93 核 |
| 峰值 RSS | 2.1MiB | 1.76GiB |
| 上下文切换 | 80 | 16,399 |
| 创建线程 | 0 | 10,000 |

两个程序输出完全一致。

**主要原因**

1. [multi_thread.rs:293](/home/maji/Projects/1brc/src/multi_thread.rs:293) 创建了 **10GB BufReader**。

实际文件只有 1.38GB，因此几次 `read` 就把整个文件读进用户态缓冲区，并一直保留到函数结束。这直接贡献了约 1.28GiB RSS。baseline 的默认 8KB 缓冲只需要约 2MiB RSS，内核页缓存已经能处理顺序读取，不需要应用层缓存整个文件。

2. [multi_thread.rs:295](/home/maji/Projects/1brc/src/multi_thread.rs:295) 每 10,000 行创建一个 OS 线程。

1 亿行正好创建 **10,000 个线程**。`strace` 结果：

```text
clone3: 10,000
mmap:   20,025
munmap: 19,998
```

每个任务平均只有约 138KB 输入，线程创建、栈映射、调度和销毁成本已经接近甚至超过实际解析工作。

3. 并行度实际上非常低。

虽然机器有 20 个逻辑 CPU，但 `maji` 平均只使用 1.93 核。主线程必须串行执行：

- 扫描换行
- 为每一行创建 `String`
- 每 10,000 行创建线程
- 最后串行合并所有结果

所以它不是 20 核并行解析，而更接近“一个生产线程加零散短任务”。

4. 结果 channel 导致大量 HashMap 堆积。

线程在 [multi_thread.rs:324](/home/maji/Projects/1brc/src/multi_thread.rs:324) 发送局部 map，但主线程直到整个文件读取完毕后，才在 [multi_thread.rs:352](/home/maji/Projects/1brc/src/multi_thread.rs:352) 开始接收。

每个 chunk 基本包含全部 413 个城市，因此最多产生：

```text
10,000 maps × 413 entries ≈ 413 万个聚合项
```

而 baseline 从始至终只有 413 个聚合项。这基本解释了除文件缓冲外剩余的约 500MiB 内存。

5. 重复聚合工作非常多。

baseline 每行直接更新最终 map。`maji` 则先更新 10,000 个局部 map，再串行合并约 413 万个条目。多线程不仅没有减少工作，反而增加了大量哈希、字符串分配和归并。

6. [multi_thread.rs:309](/home/maji/Projects/1brc/src/multi_thread.rs:309) 的 `mem::take` 丢弃了预分配容量。

第一批使用 `Vec::with_capacity(10000)`，之后每一批都从容量 0 开始扩容，产生额外 realloc。它不是主因，但会继续增加分配成本。

**正确的优化方向**

应该使用固定的约 20 个工作线程，而不是 10,000 个线程：

- 使用 1–8MiB BufReader，或者直接 mmap 文件。
- 按换行边界将文件划分为约 20 个大区间。
- 每个线程只维护一个局部 HashMap。
- 最终只合并约 20 个 map。
- 直接解析字节，避免 `lines()` 为 1 亿行创建 `String`。
- 温度按整数十分位解析，避免 `parse::<f64>()`。
- 流式实现则使用有界 channel，限制待处理 chunk 数量。

这样局部聚合项会从约 413 万降到约 8,260，并且可以真正利用全部 CPU。当前慢的核心不是 Rust 多线程性能，而是任务划分和数据生命周期设计。