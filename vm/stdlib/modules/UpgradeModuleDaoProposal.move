address 0x1 {
  module UpgradeModuleDaoProposal {
    use 0x1::PackageTxnManager;
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::Option;
    use 0x1::Dao;
    use 0x1::Block;

    resource struct UpgradeModuleCapabilities<TokenT> {
      caps: vector<WrappedUpgradePlanCapability>,
    }

    resource struct WrappedUpgradePlanCapability {
      cap: PackageTxnManager::UpgradePlanCapability,
    }

    // const UPGRADE_DELAY: u64 = 200;

    struct UpgradeModule {
      module_address: address,
      module_hash: vector<u8>,
    }

    public fun plugin<TokenT>(signer: &signer) {
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);

      let caps = UpgradeModuleCapabilities<TokenT> {
        caps: Vector::empty(),
      };
      move_to(signer, caps)
    }

    /// If this govverment can upgrade module, call this to register capability.
    public fun delegate_module_upgrade_capability<TokenT>(signer: &signer, cap: PackageTxnManager::UpgradePlanCapability)
    acquires UpgradeModuleCapabilities {
      let token_issuer = Token::token_address<TokenT>();
      assert(Signer::address_of(signer) == token_issuer, 401);

      let caps = borrow_global_mut<UpgradeModuleCapabilities<TokenT>>(token_issuer);
      // TODO: should check duplicate cap?
      // for now, only one cap exists for a module address.
      Vector::push_back(&mut caps.caps, WrappedUpgradePlanCapability {cap});
    }

    /// check whether this gov has the ability to upgrade module in `moudle_address`.
    public fun able_to_upgrade<TokenT>(module_address: address): bool acquires UpgradeModuleCapabilities {
      let pos = find_module_upgrade_cap<TokenT>(module_address);
      Option::is_some(&pos)
    }

    /// propose a module upgrade, called by proposer.
    public fun propose_module_upgrade<TokenT: copyable>(signer: &signer, module_address: address, module_hash: vector<u8>)
    acquires UpgradeModuleCapabilities {
      assert(able_to_upgrade<TokenT>(module_address), 400);
      Dao::propose<TokenT, UpgradeModule>(signer, UpgradeModule {
        module_address,
        module_hash
      }, 200); // TODO: replace 200 with DAO::MIN_ACTION_DELAY
    }

    public fun submit_module_upgrade_plan<TokenT: copyable>(_signer: &signer, proposer_address: address, proposal_id: u64)
    acquires UpgradeModuleCapabilities {
      let UpgradeModule {
        module_address,
        module_hash
      } = Dao::extract_proposal_action<TokenT, UpgradeModule>(proposer_address, proposal_id);
      let pos = find_module_upgrade_cap<TokenT>(module_address);
      assert(Option::is_some(&pos), 500);
      let pos = Option::extract(&mut pos);
      let caps = borrow_global<UpgradeModuleCapabilities<TokenT>>(Token::token_address<TokenT>());
      let cap = Vector::borrow(&caps.caps, pos);
      PackageTxnManager::submit_upgrade_plan_with_cap(&cap.cap, module_hash, Block::get_current_block_number());
    }

    fun find_module_upgrade_cap<TokenT>(module_address: address): Option::Option<u64> acquires UpgradeModuleCapabilities {
      let token_issuer = Token::token_address<TokenT>();
      let caps = borrow_global<UpgradeModuleCapabilities<TokenT>>(token_issuer);
      let cap_len = Vector::length(&caps.caps);
      let i = 0;
      while (i < cap_len) {
        let cap = Vector::borrow(&caps.caps, i);
        let account_address = PackageTxnManager::account_address(&cap.cap);
        if (account_address == module_address) {
          return Option::some(i)
        };
        i = i + 1;
      };
      Option::none<u64>()
    }
  }
}
