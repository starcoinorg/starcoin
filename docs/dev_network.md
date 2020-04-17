# Start a starcoin dev network

### Startup dev node and init account

1. Please [install](./install.md) or [build](./buid.md) starcoin first.

2. Run first local dev node, execute the following command to start a starcoin node and console:

   ```shell
   starcoin -n dev console
   ``` 
3. Show wallet accounts by execute the following command in starcoin console:
   
   ```shell
   wallet list 
   ```
4. Show account detail by command:

    ```shell
   wallet show $your_account_address
   ```
   
   Because the account is not create on chain, so balance is empty.

5. Get some coin for default account (this command only work for dev net.)
  
   ```shell
   dev get_coin
   ```
   ​	
   Run wallet show your account address again, your account should already have some coins.

### Transfer coin to another account: 

1. Create a new account by command:

    ```shell
    wallet create -p $your_wallet_password
    ```
2. Unlock your default account by command:

    ```shell
    wallet unlock $your_account_address -p
    ```
    The default account password is empty string. 
3. Transfer coin to new account by command:
    ```shell
    txn transfer -f $your_default_account_address -t $your_new_account_address -k $your_new_account_public_key -v $transfer_amount
    ```
### Startup multi dev node to build a network

1. Get your first node address by command in starcoin console:

    ```shell
    node info
   ```

   The result's self_address is your node p2p peer address, copy the address.

2. Execute the following command to start a new node and connect to first node.

    ```shell
    starcoin -n dev --seed $first_node_address console
    ```
3. Execute the following command in starcoin console to see peers info:

    ```shell
    node peers
    ```
   
   The two node should know each other now.

### Note

1. If you just want to start a node without connecting to the console, just remove the `console` subcommand.

2. The dev node use random port and temp data dir, and will clear the data after node process exist. If you want to reuse data, please add -d/--data-dir to node start command, such as:
    ```shell
   ​starcoin -n dev -d /data/starcoin console
    ```
3. If you want connect node from another compute, please change the `127.0.0.1` of `self_address` to a right ip address.

Next [Connect to test network](./test_network.md) or [Deploy Move contract](./deploy_move_contract.md).