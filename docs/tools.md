可以按“先粗后细”的顺序使用这些工具。

**1. `/usr/bin/time -v`：先确认问题类型**

```bash
/usr/bin/time -v target/release/maji > /dev/null
```

重点看：

- `Elapsed`：实际耗时
- `User time + System time`：总 CPU 工作量
- `Percent of CPU`：并行度，`600%` 约等于 6 核
- `Maximum resident set size`：峰值内存
- `Minor page faults`：频繁内存分配或触页

**2. `perf stat`：硬件计数器**

```bash
perf stat -e \
task-clock,context-switches,cpu-migrations,page-faults,\
cycles,instructions,branches,branch-misses,cache-misses \
target/release/maji > /dev/null
```

常见判断：

- `task-clock / elapsed`：平均使用多少核
- instructions 大幅增加：做了太多额外工作
- context-switches 高：线程过多或锁竞争
- cache-misses 高：数据结构或访问模式不友好
- page-faults 高：大量内存分配、初始化

**3. Flamegraph：找 CPU 时间花在哪里**

项目已有命令：

```bash
just fg maji
```

或者：

```bash
cargo flamegraph --release --bin maji -- > /dev/null
```

图中横向越宽表示累计 CPU 时间越多。重点寻找：

- `alloc`、`dealloc`
- `String`、`Vec` 扩容
- `HashMap`
- `BufRead::read_line`
- `parse::<f64>`
- mutex/channel
- 线程创建函数

注意：Flamegraph 显示的是 CPU 消耗，不一定直接显示主线程等待或低并行度。

**4. `perf record/report`：比火焰图更灵活**

```bash
perf record -g target/release/maji > /dev/null
perf report
```

查看不同线程、调用栈以及函数占比。

**5. `strace -c`：检查系统调用**

```bash
strace -f -c \
  -e trace=read,mmap,munmap,clone,clone3,futex \
  target/release/maji > /dev/null
```

本次问题中它能直接发现：

- 创建了多少线程
- `read` 调用次数
- 大量内存映射
- `futex` 锁等待

不要用未汇总的完整 `strace -f` 跑大型输入，输出会非常大。

**6. Heaptrack：分析内存分配**

```bash
heaptrack target/release/maji > /dev/null
heaptrack_gui heaptrack.maji.*
```

可以定位：

- 哪个调用栈分配最多
- 峰值内存由什么对象组成
- 分配次数和临时对象数量

对于 `Vec<String>`、每行创建 `String` 这类问题尤其有效。

**7. 并行时间线工具**

可以使用：

```bash
perf sched record target/release/maji
perf sched timehist
```

或者使用 Tracy。它们适合观察：

- 工作线程何时启动
- 是否只有少数线程工作
- 工作负载是否均衡
- 是否频繁等待锁或 channel

**推荐诊断流程**

```text
release 构建
  ↓
time -v：耗时、CPU、RSS
  ↓
perf stat：工作量、并行度、缓存、切换
  ↓
flamegraph：定位热点函数
  ↓
strace / heaptrack / perf sched：针对系统调用、内存或调度深入分析
  ↓
修改后重新测量，并和正确输出 cmp
```

基准测试时应直接运行构建产物，避免把 Cargo 的开销算进去：

```bash
cargo build --release --bin maji
/usr/bin/time -v target/release/maji > /tmp/maji.out
cmp /tmp/maji.out /tmp/baseline.out
```