// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#include "lcs.hpp"
#include "starcoin_stdlib.hpp"
#include "starcoin_types.hpp"
#include "serde.hpp"
#include <memory>

using namespace starcoin_stdlib;
using namespace starcoin_types;
using namespace serde;

int main() {
    auto token = TypeTag{TypeTag::Struct{StructTag{
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
        {"LBR"},
        {"LBR"},
        {},
    }}};
    auto payee = AccountAddress{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                                0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22};
    uint128_t amount = uint128_t {0,1234567};
    auto script =
        encode_peer_to_peer_with_metadata_script(token, payee, {},amount, {});

    auto serializer = LcsSerializer();
    Serializable<Script>::serialize(script, serializer);
    auto output = std::move(serializer).bytes();
    for (uint8_t o : output) {
        printf("%d ", o);
    };
    printf("\n");
    return 0;
}
