/// The module for the Treasury of DAO, which can hold the token of DAO.
spec starcoin_framework::treasury {

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    spec Treasury {
        // invariant [abstract] balance.value <= coin::spec_abstract_total_value<TokenT>();
    }

    spec initialize {
        use std::signer;

        aborts_if signer::address_of(signer) != @0x2;
        aborts_if exists<Treasury<TokenT>>(@0x2);
        ensures exists<Treasury<TokenT>>(@0x2);
        ensures result == WithdrawCapability<TokenT> {};
    }

    spec exists_at {
        aborts_if false;
        ensures result == exists<Treasury<TokenT>>(@0x2);
    }


    spec balance {
        aborts_if false;
        ensures if (exists<Treasury<TokenT>>(@0x2))
            result == spec_balance<TokenT>()
        else
            result == 0;
    }



    spec deposit {
        aborts_if !exists<Treasury<TokenT>>(@0x2);
        aborts_if spec_balance<TokenT>() + token.value > MAX_U128;
        ensures spec_balance<TokenT>() == old(spec_balance<TokenT>()) + token.value;
    }


    spec do_withdraw {
        include WithdrawSchema<TokenT>;
    }

    spec schema WithdrawSchema<TokenT> {
        amount: u64;

        aborts_if amount <= 0;
        aborts_if !exists<Treasury<TokenT>>(@0x2);
        aborts_if spec_balance<TokenT>() < amount;
        ensures spec_balance<TokenT>() == old(spec_balance<TokenT>()) - amount;
    }


    spec withdraw_with_capability {
        include WithdrawSchema<TokenT>;
    }


    spec withdraw {
        aborts_if !exists<WithdrawCapability<TokenT>>(signer::address_of(signer));
        include WithdrawSchema<TokenT>;
    }


    spec issue_linear_withdraw_capability {
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;

        aborts_if period == 0;
        aborts_if amount == 0;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
    }


    spec withdraw_with_linear_capability {
        pragma aborts_if_is_partial;
        // TODO: See [MUL_DIV]
        // include WithdrawSchema<TokenT> {amount: ?};
    }


    spec withdraw_by_linear {
        use std::signer;

        pragma aborts_if_is_partial;
        aborts_if !exists<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
        // TODO: See [MUL_DIV]
        // include WithdrawSchema<TokenT> {amount: ?};
    }


    spec split_linear_withdraw_cap {
        pragma aborts_if_is_partial;
        ensures old(cap.total - cap.withdraw) ==
            result_1.value + (result_2.total - result_2.withdraw) + (cap.total - cap.withdraw);
    }

    spec withdraw_amount_of_linear_cap {
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;
        use starcoin_framework::math128;


        // TODO: [MUL_DIV] The most important property is the amount of value.
        //        However, Math::mul_div remains to be uninterpreted
        pragma aborts_if_is_partial;
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if timestamp::spec_now_seconds() < cap.start_time;
        aborts_if timestamp::spec_now_seconds() - cap.start_time >= cap.period && cap.total < cap.withdraw;
        aborts_if [abstract]
            timestamp::spec_now_seconds() - cap.start_time < cap.period && math128::spec_mul_div() < cap.withdraw;
        ensures [abstract] result <= cap.total - cap.withdraw;
    }


    spec is_empty_linear_withdraw_cap {
        aborts_if false;
        ensures result == (key.total == key.withdraw);
    }

    // Improvement: Make move prover support the following definition.
    // Following specs contains lots of duplication
    //spec schema AddCapability<Capability> {
    //    signer: signer;

    //    aborts_if exists<Capability>(signer::address_of(signer));
    //    ensures exists<Capability>(signer::address_of(signer));
    //}


    spec remove_withdraw_capability {
        use std::signer;

        aborts_if !exists<WithdrawCapability<TokenT>>(signer::address_of(signer));
        ensures !exists<WithdrawCapability<TokenT>>(signer::address_of(signer));
    }

    spec add_withdraw_capability {
        use std::signer;
        aborts_if exists<WithdrawCapability<TokenT>>(signer::address_of(signer));
        ensures exists<WithdrawCapability<TokenT>>(signer::address_of(signer));
    }

    spec destroy_withdraw_capability {
    }


    spec add_linear_withdraw_capability {
        use std::signer;
        aborts_if exists<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
        ensures exists<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
    }


    spec remove_linear_withdraw_capability {
        use std::signer;
        aborts_if !exists<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
        ensures !exists<LinearWithdrawCapability<TokenT>>(signer::address_of(signer));
    }


    spec fun spec_balance<TokenType>(): num {
        global<Treasury<TokenType>>(@0x2).balance.value
    }

}