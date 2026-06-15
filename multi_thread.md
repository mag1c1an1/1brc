# bad1
```rust
    pub fn __main() {
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
结果验证了主要瓶颈：maji 用了 17.05s，但累计 CPU 时间约 33s，平均只利用 1.93
  个核；峰值 RSS 是 1,849,652KiB，约 1.76GiB。也就是说它付出了多线程成本，却没有
  形成持续并行

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