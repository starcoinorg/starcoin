## Contrib Contracts

The repo contains contribution move libraries, which are not suitable to be in Starcoin stdlib.

The rust codes are needed to run test for these modules, underlying it use starcoin-executor to do the heavy job.


### Test

Just run:

```shell
cargo test
```

### Modules

- MerkleDistributor.move: airdrop contract on Starcoin. see `cmd/merkle-distributor` for more details.
