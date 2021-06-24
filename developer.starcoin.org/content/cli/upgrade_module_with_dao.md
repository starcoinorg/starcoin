---
title: Upgrade module with Dao
weight: 14
---

Starcoin support upgrade module by DAO.

<!--more-->

1. compile the module which you want to upgrade.
2. unlock your account.

```bash
starcoin% account unlock <account address>
```

3. submit upgrade module proposal with your account.

```bash
starcoin% dev module-proposal -s <account address> -m <module path> -v <version>
```

4. query proposal state.

```bash
starcoin% dev call --module-address <module address> --module-name Dao --func-name proposal_state -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModule --arg <proposal address> --arg <proposal id>
```

5. anyone can vote proposal when the proposal state is ACTIVE.
```bash
starcoin% dev execute-function -s <account address> -b --function 0x1::DaoVoteScripts::cast_vote -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModule --arg <proposal address> --arg <proposal id> --arg <agree> --arg <votes>u128
```

6. anyone can queue proposal when the proposal state is AGREED.
```bash
starcoin% dev module-queue -s <account address> -a <proposal address> -i <proposal id>
```

7. anyone can submit plan when the proposal state is QUEUED.
```bash
starcoin% dev module-plan -s <account address> -a <proposal address> -i <proposal id>
```

8. anyone can upgrade the module when the proposal state is EXECUTABLE.
```bash
starcoin% dev module-exe -s <account address> -m <module path>
```