#Benchmark

1. run all benchmark

```shell
cargo bench
```

2. run a special benchmark

```shell
cargo bench --bench bench_state_tree
```

3. run a special benchmark with pprof (on linux)
```shell
cargo bench --bench bench_state_tree -- --profile-time=10
```
Notice:
run benchmark with pprof on macOS show this error, linux(ubuntu) run successful
```shell
error: process didn't exit successfully: `./bench_state_tree-3daab626ee8d6834 --profile-time=10 --bench` (signal: 11, SIGSEGV: invalid memory reference)
```