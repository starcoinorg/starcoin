# Proxima Sync Compatibility Investigation

## Goal

Build a controlled compatibility test for `proxima` sync:

- control: `old -> old`
- experiment: `new -> old`

The purpose is to determine whether the current branch has a backward-compatibility regression in `network-p2p` request-response when syncing from an older `proxima` peer.

## Scope

- Reuse existing local binaries and data directories when possible.
- Avoid repeated cold starts unless a fresh data dir is required to prove a point.
- Keep the old peer as the only seed for the new node, so the peer-version variable stays isolated.

## Versions

### Old peer

- commit: `a9b9331c8`
- role: reference `proxima` peer known to sync successfully

### New node

- repo: `/Users/simon/starcoin-projects/starcoin`
- branch: `refactor-with-jsonrpsee-v2`
- role: node under test

## Paths

### Old version assets

- source worktree: `/private/tmp/starcoin-proxima-a9b9331c8`
- binary: `/Users/simon/starcoin-projects/target-proxima-a9b9331c8/debug/starcoin`
- synced data dir: `/tmp/starcoin-proxima-sync-a9b9331c8-20260316-1612`

### New version assets

- source worktree: `/Users/simon/starcoin-projects/starcoin`
- binary: `/Users/simon/starcoin-projects/starcoin/target/debug/starcoin`

### Test logs and runtime dirs

- tracker: `docs/proxima-sync-compat-investigation.md`
- test root: `/tmp/starcoin-proxima-compat`

## Test Matrix

### Control: old -> old

Use one old-version node as the only seed, then sync another old-version node from it.

Expected result:

- sync should make progress
- `get_block_ids` and `get_block_infos` should both succeed
- no repeatable `ConnectionClosed` / `NotConnected` loop during block sync bootstrap

### Experiment: new -> old

Use the same old-version seed, then sync one current-branch node from it.

Expected result:

- if sync fails while control succeeds, the regression is in the new node
- if failure begins after initial request-response success, focus on `network-p2p`

## Common Startup Shape

All test nodes should:

- use `-n proxima`
- disable HTTP/TCP/WebSocket/IPC RPC
- disable stratum
- disable metrics
- use explicit `--listen` ports to avoid collisions
- use explicit `--seed` / `--disable-seed` to isolate the peer set

## Planned Commands

### Old seed

```bash
OLD_BIN=/Users/simon/starcoin-projects/target-proxima-a9b9331c8/debug/starcoin
OLD_SEED_DIR=/tmp/starcoin-proxima-sync-a9b9331c8-20260316-1612

"$OLD_BIN" \
  -n proxima \
  -d "$OLD_SEED_DIR" \
  --listen /ip4/127.0.0.1/tcp/19840 \
  --disable-http-rpc \
  --disable-tcp-rpc \
  --disable-websocket-rpc \
  --disable-ipc-rpc \
  --disable-stratum \
  --disable-metrics true \
  --disable-seed
```

### Old leecher

```bash
OLD_LEECH_DIR=/tmp/starcoin-proxima-compat/old-leecher
OLD_SEED_ADDR=<fill from old seed log or node info>

"$OLD_BIN" \
  -n proxima \
  -d "$OLD_LEECH_DIR" \
  --listen /ip4/127.0.0.1/tcp/19841 \
  --disable-http-rpc \
  --disable-tcp-rpc \
  --disable-websocket-rpc \
  --disable-ipc-rpc \
  --disable-stratum \
  --disable-metrics true \
  --disable-seed \
  --seed "$OLD_SEED_ADDR"
```

### New leecher

```bash
NEW_BIN=/Users/simon/starcoin-projects/starcoin/target/debug/starcoin
NEW_LEECH_DIR=/tmp/starcoin-proxima-compat/new-leecher

"$NEW_BIN" \
  -n proxima \
  -d "$NEW_LEECH_DIR" \
  --listen /ip4/127.0.0.1/tcp/19842 \
  --disable-http-rpc \
  --disable-tcp-rpc \
  --disable-websocket-rpc \
  --disable-ipc-rpc \
  --disable-stratum \
  --disable-metrics true \
  --disable-seed \
  --seed "$OLD_SEED_ADDR"
```

## Execution Log

### 2026-03-16 Initial setup

- Created this tracker.
- Reusing the existing old-version binary and synced old data dir.
- Intention: keep the old synced node warm and use it as the only seed for both control and experiment runs.

### 2026-03-16 Seed bootstrap

- Started the old-version seed in a persistent PTY session instead of `nohup`, because closing stdin shuts the CLI down and stops the node.
- Final old-seed command shape:
  - old binary
  - `-n proxima`
  - `--listen /ip4/127.0.0.1/tcp/28440`
  - `--rpc-address 127.0.0.1 --http-port 28550`
  - RPC endpoints other than HTTP disabled
  - metrics/stratum disabled
- Observed self address:
  - `/ip4/127.0.0.1/tcp/28440/p2p/12D3KooWCp2E6VoUsSmmUUy6pRwGhiKk2Su57XzCmbdxKGHH2SMb`
- Important note:
  - the reused old data dir was not already at a high finalized height
  - after restart with builtin seeds enabled, the old node connected to 3 `proxima` peers and began syncing again
  - this old node is now the dedicated seed for the local compatibility tests

### 2026-03-16 Control run: old -> old

- Started a fresh old-version leecher with:
  - `--listen /ip4/127.0.0.1/tcp/28441`
  - `--http-port 28551`
  - `--disable-seed`
  - `--seed /ip4/127.0.0.1/tcp/28440/p2p/12D3KooWCp2E6VoUsSmmUUy6pRwGhiKk2Su57XzCmbdxKGHH2SMb`
- This first design was wrong.
- Reason:
  - `--disable-seed` clears *all* seeds, including explicitly supplied `--seed`
  - this is confirmed by [`config/src/network_config.rs`](/Users/simon/starcoin-projects/starcoin/config/src/network_config.rs), where `seeds()` returns `[]` immediately when `disable_seed` is true
- Observed effect:
  - the old leecher logged `Final bootstrap seeds: []`
  - then failed with `No peers to sync`

### 2026-03-16 Experiment start: new -> old

- Started a fresh new-version leecher with:
  - `--listen /ip4/127.0.0.1/tcp/28442`
  - `--http-port 28552`
  - `--disable-seed`
  - same old seed address as the control
- New binary under test:
  - `starcoin 2.1.1 (build:proxima-dual-verse-dag-03-140-g22662805c-dirty)`
- Important note:
  - this binary is from a dirty worktree
  - uncommitted files:
    - `network-p2p/src/request_responses.rs`
    - `network-p2p/src/service.rs`
    - `network-p2p/src/transport.rs`
- Current experiment status:
  - the fresh data dir was slow because of first-start migration
  - switched to the already-initialized new data dir `/tmp/starcoin-proxima-sync-head-fresh-20260316-1618` to avoid repeated cold-start cost

### 2026-03-16 Revised single-peer design

- Updated the design to:
  - start both leechers with `--disable-seed`
  - expose `network_manager` over HTTP via `--http-apis chain,node,network_manager`
  - start with zero bootstrap peers
  - inject the old local seed after startup through `network_manager.add_peer`
- This preserves the single-peer test shape without reintroducing builtin public seeds during bootstrap.

### 2026-03-16 Revised control result: old -> old(local)

- Manual peer injection succeeded:
  - `network_manager.add_peer("/ip4/127.0.0.1/tcp/28440/p2p/12D3KooWCp2E6VoUsSmmUUy6pRwGhiKk2Su57XzCmbdxKGHH2SMb")`
- After injection, the old leecher immediately entered sync.
- Key log sequence:
  - `Find target ...`
  - `start full sync task`
  - successful `/starcoin/rpc/get_block_ids`
  - successful `/starcoin/rpc/get_block_infos`
- Representative lines:
  - `20:52:39.065630` `get_block_ids ... true`
  - `20:52:39.081850` `call method: "get_block_infos"`
  - `20:52:39.084004` `get_block_infos ... true`

### 2026-03-16 Revised experiment result: new -> old(local)

- Manual peer injection also succeeded for the new node.
- The new node then entered sync successfully against the local old-version seed.
- Key log sequence:
  - `20:52:39.106194` `start full sync task`
  - `20:52:39.122000`-ish successful `get_block_ids`
  - `20:52:39.205834` `call method: "get_block_infos"`
  - `20:52:39.207931` `get_block_infos ... true`
- Verified runtime progress:
  - `20:53:29` `chain.info` showed head number `49`
  - later `chain.info` showed head number `146`
  - the log continuously reported `sync processs complete a block execution`

### 2026-03-16 Narrowed conclusion

- The failure does **not** reproduce in the controlled `new -> old(local)` case.
- Therefore the earlier conclusion needs refinement:
  - this is not a blanket “new request-response cannot talk to old peers” failure
  - the incompatibility appears to depend on the specific remote `proxima` peer environment or connection conditions
- What is now disproven:
  - “all old-version peers are incompatible with the new branch”
  - “`get_block_infos` always fails against old peers”
- What is still consistent with the evidence:
  - some public `proxima` peers reset the connection during the request-response flow
  - the local old seed does not
  - the issue is therefore likely tied to a narrower interoperability condition than simple version skew

## Findings

### Current working hypothesis

- The regression is not in `sync` business logic.
- The earlier “broad old/new incompatibility” hypothesis is too strong.
- The strongest current evidence points to a narrower, peer- or connection-specific failure mode on public `proxima` peers:
  - a public peer may reset the connection during `get_block_ids`
  - but the same binary can later sync successfully and complete both `get_block_ids` and `get_block_infos`
  - the same public peer id may fail once and later succeed
- This shifts the focus away from a blanket request-response compatibility break and toward:
  - transient public-peer behavior
  - connection lifecycle differences on WAN peers
  - or a narrower interoperability bug that does not trigger in the local-old setup

### Controlled-environment findings so far

- The local single-peer environment is valid for diagnosis after switching from `--seed` bootstrap to runtime `network_manager.add_peer`.
- `old -> old(local)` succeeds.
- `new -> old(local)` also succeeds.
- This rules out a coarse-grained old/new protocol mismatch.

### Public-peer observations after the controlled test

- The reused new-node data dir continued syncing after the controlled `new -> old(local)` experiment.
- Current observed head from `chain.info` on the new node:
  - head number `1687`
- Current connected peers include:
  - the local old seed `12D3KooWCp2E6VoUsSmmUUy6pRwGhiKk2Su57XzCmbdxKGHH2SMb`
  - public `proxima` peers `12D3KooWS6QFSSPT9KMC5tkG3aWNyNEzM5TpJkAfPedzLGQfecvM`
  - public `proxima` peers `12D3KooWAkg1htBrpZ5tyoeMt4UMJgsS89uPeJnR2XkTsfzjh3ph`
  - public `proxima` peers `12D3KooWExj12zhczTnswmBvgwvzT9nTaurfogmt5Nq1bz7MVXtm`
- Important refinement:
  - the earlier public-failure window showed `12D3KooWEx...` resetting the connection during `get_block_ids`
  - later in the same new-node log, that same peer successfully served `get_block_ids`
  - later in the same new-node log, that same peer also successfully served `get_block_infos`
- Therefore the current evidence does not support “peer `12D3KooWEx...` is categorically incompatible with the new branch”.

### Fresh-dial vs later-success contrast

- In the failure window at `17:00:09`:
  - peer `12D3KooWEx...` is newly connected
  - sync immediately selects it as the target
  - the very first outbound `get_block_ids` is sent
  - the connection is then reset by the remote side before a successful response is recorded
- In the later-success window at `20:59:12`:
  - the same peer id successfully serves `get_block_ids`
  - the same peer id successfully serves `get_block_infos`
  - block sync continues into accumulator sync
- This makes the timing and connection lifecycle more suspicious than the RPC method itself:
  - the failure is consistent with “first use on a fresh/public connection can be reset”
  - the later success is consistent with “the peer is usable once the session is established or when a different connection attempt succeeds”

### Code-path finding: sync starts from notification open, not RPC readiness

- `sync/src/sync.rs` subscribes to `PeerEvent`.
- On `PeerEvent::Open`, it immediately runs:
  - `ctx.notify(CheckSyncEvent::default())`
- `network/src/service.rs` emits `PeerEvent::Open` from `Event::NotificationStreamOpened`.
- The same file has an explicit comment:
  - every notification stream open currently triggers a `PeerEvent`
  - this means the event can be repeated per notification protocol
- `network/src/service_ref.rs` sends raw RPC as soon as:
  - the peer exists in local peer state
  - the cached peer info says the RPC path is supported
- There is no extra gate there to wait for a “request-response ready and stable” signal.

Interpretation:

- A fresh public peer can become eligible for sync as soon as the notification stream opens and status advertises the RPC methods.
- `sync` can then send `get_block_ids` almost immediately after the fresh connection appears.
- In the failing public window, this happened within about 1 ms of the peer open sequence.
- In the successful later window, the same peer was already established and then served both `get_block_ids` and `get_block_infos`.
- This makes an “early use of a fresh peer connection” race a stronger hypothesis than a generic `get_block_infos` incompatibility.

## Next actions

1. Validate the timing hypothesis with a narrow experiment:
   - delay sync checking after `PeerEvent::Open`
   - or gate sync peer eligibility on a small connection-age threshold
2. Compare whether that removes the fresh-dial public failure while preserving later sync progress.
3. Only after that, reassess whether a compatibility fallback in `network-p2p` is still needed.

## Validation Experiment

### 2026-03-16 Delayed `CheckSyncEvent` on peer open

- Temporary experiment:
  - changed `sync/src/sync.rs`
  - on `PeerEvent::Open`, replaced immediate `ctx.notify(CheckSyncEvent::default())`
  - with `ctx.run_later(Duration::from_millis(500), ...)`
- Rebuilt only the CLI binary:
  - `cargo build -p starcoin-cmd --bin starcoin`
- Reused the already-initialized new-node data dir:
  - `/tmp/starcoin-proxima-sync-head-fresh-20260316-1618`
- Restarted the new node with:
  - `--disable-seed`
  - `--http-apis chain,node,network_manager`
- Manually injected the previously failing public peer:
  - `/ip4/146.190.200.229/tcp/9840/p2p/12D3KooWExj12zhczTnswmBvgwvzT9nTaurfogmt5Nq1bz7MVXtm`

### Result of the delayed-open experiment

- In this run, the fresh-dial request to `12D3KooWEx...` succeeded:
  - `21:06:50.239128` first `get_block_ids`
  - `21:06:50.529540` `get_block_ids ... true`
  - `21:06:50.735991` second `get_block_ids ... true`
- There was no immediate `ConnectionClosed` / `Connection reset by peer` for that peer in this window.
- This does not prove the delay is the full fix.
- But it does strengthen the timing hypothesis:
  - delaying sync peer use after `PeerEvent::Open` appears to reduce the fresh-dial failure mode
  - the original failure is therefore more likely tied to connection readiness / early peer use than to a stable RPC incompatibility

### 2026-03-16 Minimal-fix retest after removing diagnostic changes

- Removed the unrelated diagnostic changes from `network-p2p`.
- Kept only the `sync/src/sync.rs` delay on `PeerEvent::Open`.
- Rebuilt:
  - `cargo build -p starcoin-cmd --bin starcoin`
- Reused the same initialized data dir and the same single-peer injection target:
  - peer `12D3KooWExj12zhczTnswmBvgwvzT9nTaurfogmt5Nq1bz7MVXtm`

Result:

- The minimal-fix build also succeeded on the previously problematic fresh-dial path.
- Representative sequence:
  - `21:28:59.052708` first `get_block_ids`
  - `21:29:01.084654` `get_block_ids ... true`
  - `21:29:01.645476` `get_block_infos`
  - `21:29:01.943312` `get_block_infos ... true`
- No immediate `ConnectionClosed` or `Connection reset by peer` appeared in this retest window.

Current conclusion:

- The smallest validated fix so far is to delay sync checking slightly after `PeerEvent::Open`.
- The larger `network-p2p` transport/request-response diagnostic edits were not required to reproduce the improvement and should not be kept.

### 2026-03-16 Refined fix: deduplicated peer debounce

- The plain fixed-delay version was still too blunt:
  - every `PeerEvent::Open` schedules a delayed check
  - `network/src/service.rs` can emit repeated `PeerEvent::Open` entries because notification streams open per protocol
- Refined implementation in `sync/src/sync.rs`:
  - add a `pending_peer_sync_checks` set
  - on `PeerEvent::Open`, only schedule a delayed `CheckSyncEvent` if that peer is not already pending
  - on `PeerEvent::Close`, remove the peer from the pending set
- This keeps the protection for fresh peers while avoiding repeated delayed sync checks for the same peer open burst.

### Validation of the refined fix

- Rebuilt the refined implementation:
  - `cargo build -p starcoin-cmd --bin starcoin`
- Reused the same initialized data dir and the same problematic public peer:
  - peer `12D3KooWExj12zhczTnswmBvgwvzT9nTaurfogmt5Nq1bz7MVXtm`
- Fresh-dial result still succeeded:
  - `21:32:18.906295` first `get_block_ids`
  - `21:32:19.625869` `get_block_ids ... true`
  - `21:32:20.209949` `get_block_infos`
  - `21:32:20.501950` `get_block_infos ... true`

Updated conclusion:

- The better fix is not “blind sleep on every open”.
- The better fix is a deduplicated debounce for fresh peers in `SyncService`.
- This keeps the fix narrowly scoped to sync peer eligibility and avoids retaining unrelated `network-p2p` experiments.
