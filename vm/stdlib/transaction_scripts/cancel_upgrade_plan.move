script {
    use 0x1::PackageTxnManager;

    fun cancel_upgrade_plan(
        signer: &signer,
    ) {
        PackageTxnManager::cancel_upgrade_plan(signer);
    }

    spec fun cancel_upgrade_plan {
        pragma verify = false;
    }
}
