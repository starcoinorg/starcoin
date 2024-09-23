#!/usr/bin/env bash
set +e
crates=($(cargo metadata --format-version=1 --no-deps | jq -r '.packages[].name'  | sort))

result=()
any_failed=0
for crate in "${crates[@]}"; do
    cargo check -p $crate
    if [ $? -ne 0 ]; then
        any_failed=1
        result+=("$crate\n")
    fi
done

printf $result