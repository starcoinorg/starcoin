#Benchmark

1. run all benchmark

```shell
cargo bench
```

2. run a special benchmark

```shell
cargo bench --bench bench_state_tree
```

3. run a special benchmark with pprof
```shell
cargo bench --bench bench_storage -- --profile-time=10
```