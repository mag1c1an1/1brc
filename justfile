create:
    cargo run --release --bin measurements 100000000

release-build:
    cargo build --release

bench bin="maji": release-build
    hyperfine --warmup 3 'target/release/{{bin}} > /dev/null'

fg bin="maji":
    cargo flamegraph --release --palette hot --bin={{bin}} > /dev/null

check:
    cargo run --release --bin maji > maji.ans
    diff maji.ans baseline.ans
