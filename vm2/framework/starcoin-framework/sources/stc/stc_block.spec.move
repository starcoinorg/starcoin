/// Block module provide metadata for generated blocks.
spec starcoin_framework::stc_block {
    spec initialize {
        use std::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<BlockMetadata>(signer::address_of(account));
    }

    spec get_current_block_number {
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
    }

    spec get_parent_hash {
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
    }

    spec get_current_author {
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
    }

    spec process_block_metadata {
        use std::signer;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
        aborts_if number != global<BlockMetadata>(system_addresses::get_starcoin_framework()).number + 1;
    }

    spec schema AbortsIfBlockMetadataNotExist {
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
    }


}