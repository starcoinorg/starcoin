

// Construct a ForkNodeService from given RPC address and node config.
// A RpcClient is created from the given RPC address, and be used by 
// ForkNodeService to read remote state.
//
// ForkNodeService holds a local state cache and a RpcClient.
// New blocks are stored in the local cache. 
// All states before fork_block_number are read from remote through RpcClient,
// and those after fork_block_number are read from the local cache.

pub struct ForkNodeService {
    registry: ServiceRef<RegistryService>,
}