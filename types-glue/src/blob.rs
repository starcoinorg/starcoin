// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forkable_jellyfish_merkle::blob as VM1;
use starcoin_vm2_forkable_jellyfish_merkle::blob as VM2;

pub fn vm1_to_vm2(blob: VM1::Blob) -> VM2::Blob {
    VM2::Blob::from(blob.as_ref().to_vec())
}

pub fn vm2_to_vm1(blob: VM2::Blob) -> VM1::Blob {
    VM1::Blob::from(blob.as_ref().to_vec())
}
