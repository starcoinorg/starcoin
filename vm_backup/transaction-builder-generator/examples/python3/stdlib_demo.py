# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# pyre-strict

import starcoin_types as starcoin
import serde_types as st
import lcs
import starcoin_stdlib as stdlib


def make_address(content: bytes) -> starcoin.AccountAddress:
    assert len(content) == 16
    # pyre-fixme
    return starcoin.AccountAddress(tuple(st.uint8(x) for x in content))


def main() -> None:
    token = starcoin.TypeTag__Struct(
        starcoin.StructTag(
            address=make_address(b"\x00" * 15 + b"\x01"),
            module=starcoin.Identifier("LBR"),
            name=starcoin.Identifier("LBR"),
            type_params=[],
        )
    )
    payee = make_address(b"\x22" * 16)
    amount = st.uint128(1_234_567)
    script = stdlib.encode_peer_to_peer_with_metadata_script(token, payee, b"",amount, b"")

    for b in lcs.serialize(script, starcoin.Script):
        print("%d " % b, end='')
    print()


if __name__ == "__main__":
    main()
