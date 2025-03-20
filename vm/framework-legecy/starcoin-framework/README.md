# Starcoin Move framework


## Build and Test

Install `mpm` first, download from the release page of [starcoiorg/starcoin](https://github.com/starcoinorg/starcoin).

Or use:
```bash
cargo install --git https://github.com/starcoinorg/starcoin --bin mpm
```

Build:

```shell
mpm package build 
```

Run unit test:

```shell
mpm package test
```

## Spec Test

1. Add a test to [spectests](../spectests), such as: `test_xxx.move`
2. Run the spec test `mpm spectest test_xxx.move `