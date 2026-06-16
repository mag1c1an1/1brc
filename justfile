create:
    cargo run --release --bin measurements 100000000

bench bin="maji":
    hyperfine --warmup 3 'cargo run --release --bin {{bin}} > /dev/null'

fg bin="maji":
    cargo flamegraph --release --palette hot --bin={{bin}} > /dev/null
