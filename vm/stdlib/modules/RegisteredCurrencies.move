address 0x1 {
//TODO Consider a more appropriate name.
module RegisteredCurrencies {
    use 0x1::Vector;
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct CurrencyRecord{
        // Currency module address.
        module_address: address,
        currency_code: vector<u8>,
    }

    // An on-chain config holding all of the currency codes for registered
    // currencies. Must be named "RegisteredCurrencies" for an on-chain config.
    struct RegisteredCurrencies {
        currency_codes: vector<CurrencyRecord>,
    }

    // An operations capability to allow updating of the on-chain config
    resource struct RegistrationCapability {
        cap: Config::ModifyConfigCapability<Self::RegisteredCurrencies>,
    }

    public fun initialize(account: &signer): RegistrationCapability {
        // enforce that this is only going to one specific address,
        assert(Signer::address_of(account) == singleton_address(), 0);
        let cap = Config::publish_new_config_with_capability(account, empty());

        RegistrationCapability{ cap }
    }

    fun empty(): RegisteredCurrencies {
        RegisteredCurrencies { currency_codes: Vector::empty() }
    }

    public fun add_currency_code(
        module_address: address,
        currency_code: vector<u8>,
        cap: &RegistrationCapability,
    ) {
        let config = Config::get<RegisteredCurrencies>();
        //TODO limit check cap
        let record = CurrencyRecord {module_address, currency_code};
        Vector::push_back(&mut config.currency_codes, record);
        Config::set_with_capability(&cap.cap, config);
    }

    public fun currency_records(): vector<CurrencyRecord> {
        let config = Config::get<RegisteredCurrencies>();
        *&config.currency_codes
    }

    fun singleton_address(): address {
        CoreAddresses::GENESIS_ACCOUNT()
    }

    public fun module_address_of(record: &CurrencyRecord): address{
        *&record.module_address
    }

    public fun currency_code_of(record: &CurrencyRecord): vector<u8>{
        *&record.currency_code
    }
}

}
