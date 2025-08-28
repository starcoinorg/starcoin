// Copyright © Starcoin Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

package main

import (
	"fmt"
	stdlib "testing/starcoinstdlib"
	starcoin "testing/starcointypes"
)

func demo_coin_transfer() {
	token := &starcoin.TypeTag__Struct{
		Value: starcoin.StructTag{
			Address: starcoin.AccountAddress(
				[32]uint8{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1},
			),
			Module:     starcoin.Identifier("starcoin_coin"),
			Name:       starcoin.Identifier("StarcoinCoin"),
		},
	}

	to := starcoin.AccountAddress(
		[32]uint8{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22},
	)

	amount := uint64(1_234_567)
	payload := stdlib.EncodeCoinTransfer(token, to, amount)

	call, err := stdlib.DecodeEntryFunctionPayload(payload)
	if err != nil {
		panic(fmt.Sprintf("failed to decode script: %v", err))
	}
	payment := call.(*stdlib.EntryFunctionCall__CoinTransfer)
	if payment.Amount != amount || payment.To != to {
		panic("wrong script content")
	}

	bytes, err := payload.BcsSerialize()
	if err != nil {
		panic("failed to serialize")
	}
	for _, b := range bytes {
		fmt.Printf("%d ", b)
	}
	fmt.Printf("\n")
}

func main() {
    demo_coin_transfer()
}
