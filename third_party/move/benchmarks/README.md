# Profiling MoveVM with Instrument
This doc is going to talk about how to run performance benchmarks on macOS.

## Step 1: Get latest Xcode and open Instrument

Instrument can be found in Xcode > Open Developer Tool > Instrument.
![](https://i.imgur.com/QCwJBim.png)



## Step 2: Choose a benchmark suite.

We currently have four local benchmark candidates:
- `executor_benchmark` in `diem/executor`
- `txn_bench` in `diem/language/benchmark`
- `Arith` and `call` benchmark in `diem/language/benchmark`

The first item is a comprehensive benchmark of diem adapter, executor and storage that generates a block of p2p transactions and tries to execute and commit it to the DiemDB in local storage. The second item is a benchmark of Diem adapter only with a fake executor and an in-memory storage that executes randomly generated p2p transactions. The third item, although it’s still invoking Diem adapter, is mostly testing on the MoveVM’s ability of handling simple arithmetic operations and call stacks.

## Step 3: Select the running process in Instrument.
Open instrument and create a time profiler project.

![](https://i.imgur.com/dbLht9f.png)

Launch the benchmark target in the terminal, and select it in Instrument.

![](https://i.imgur.com/LU10tZC.jpg)


## Step 4: Get analysis!

Here’s an example trace from running the benchmark

![](https://i.imgur.com/BAoprNq.jpg)
