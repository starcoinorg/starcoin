#Copyright © Starcoin Foundation
# SPDX-License-Identifier: Apache-2.0

clean:
	- rm -rf go.mod go.sum starcoinstdlib starcointypes

test:
	cargo run -p starcoin-framework release
	cargo run -p starcoin-sdk-builder -- \
		--language go --module-name starcoinstdlib \
		--package-name testing \
		--with-starcoin-types "../../../../testsuite/generate-format/tests/staged/starcoin.yaml" \
		--target-source-dir . \
		"../../../framework/starcoin-framework/build/StarcoinFramework"
	- go mod init testing
	go mod edit -replace testing=`pwd`
	go get github.com/starcoin-labs/serde-reflection/serde-generate/runtime/golang/bcs
	go get github.com/starcoin-labs/serde-reflection/serde-generate/runtime/golang/serde
	go run stdlib_demo.go

.PHONY: clean test
