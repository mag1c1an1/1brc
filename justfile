create:
    cargo run --release --bin measurements 100000000

baseline:
    time cargo run --release --bin baseline > /dev/null

start:
    time cargo run --release --bin maji > /dev/null

fg bin="baseline":
    cargo flamegraph --release --palette hot --bin={{bin}} > /dev/null
