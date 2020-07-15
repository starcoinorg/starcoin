address 0x1 {
//TODO Consider a more appropriate name.
module RegisteredCurrencies {
    use 0x1::Vector;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::SortedLinkedList::{Self, EntryHandle};

    struct CurrencyRecord{
        currency_code: vector<u8>,
    }

    public fun initialize(account: &signer) {
        // enforce that this is only going to one specific address
        assert(Signer::address_of(account) == singleton_address(), 0);
        SortedLinkedList::create_new_list<CurrencyRecord>(account, empty());
    }

    fun empty(): CurrencyRecord {
        CurrencyRecord {
            currency_code: Vector::empty()
        }
    }

    public fun add_currency_code(
        account: &signer,
        currency_code: vector<u8>,
    ) {
        let record = CurrencyRecord { currency_code: currency_code };
        SortedLinkedList::find_position_and_insert<CurrencyRecord>(account, record, currency_records());
    }

    public fun get_currency_for(addr: address, index: u64): vector<u8> {
        let entry = SortedLinkedList::entry_handle(addr, index);
        *&SortedLinkedList::get_data<CurrencyRecord>(entry).currency_code
    }

    public fun currency_records(): EntryHandle {
        SortedLinkedList::entry_handle(singleton_address(), 0)
    }

    fun singleton_address(): address {
        CoreAddresses::GENESIS_ACCOUNT()
    }


    // public fun module_address_of(record: &CurrencyRecord): address{
    //     *&record.module_address
    // }

    // public fun currency_code_of(record: &CurrencyRecord): vector<u8>{
    //     *&record.currency_code
    // }
}

}
