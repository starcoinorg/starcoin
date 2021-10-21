address Std {
module AccountScripts {
    use Std::Account;
    /// Enable account's auto-accept-token feature.
    /// The script function is reenterable.
    public(script) fun enable_auto_accept_token(account: signer) {
        Account::set_auto_accept_token(&account, true);
    }

    /// Disable account's auto-accept-token feature.
    /// The script function is reenterable.
    public(script) fun disable_auto_accept_token(account: signer) {
        Account::set_auto_accept_token(&account, false);
    }
}
}