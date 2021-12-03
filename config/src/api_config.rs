use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, PartialEq, Ord, PartialOrd, Clone, Copy, Eq, Hash)]
pub enum Api {
    Account,
    Chain,
    Debug,
    Miner,
    NetworkManager,
    NodeManager,
    Node,
    PubSub,
    State,
    SyncManager,
    TxPool,
    Contract,
}

impl Serialize for Api {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Api {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Api::from_str(&s).map_err(D::Error::custom)
    }
}

impl std::fmt::Display for Api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Self::Account => "account",
            Self::Chain => "chain",
            Self::Debug => "debug",
            Self::Miner => "miner",
            Self::NetworkManager => "network_manager",
            Self::NodeManager => "node_manager",
            Self::Node => "node",
            Self::PubSub => "pubsub",
            Self::State => "state",
            Self::SyncManager => "sync_manager",
            Self::TxPool => "txpool",
            Self::Contract => "contract",
        };
        write!(f, "{}", display)
    }
}

impl FromStr for Api {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Api::*;

        match s {
            "account" => Ok(Account),
            "chain" => Ok(Chain),
            "debug" => Ok(Debug),
            "miner" => Ok(Miner),
            "network_manager" => Ok(NetworkManager),
            "node_manager" => Ok(NodeManager),
            "node" => Ok(Node),
            "pubsub" => Ok(PubSub),
            "state" => Ok(State),
            "sync_manager" => Ok(SyncManager),
            "txpool" => Ok(TxPool),
            "contract" => Ok(Contract),
            api => Err(format!("Unknown api: {}", api)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ApiSet {
    // Unsafe context (like jsonrpc over http)
    UnsafeContext,
    // All possible APIs (safe context like token-protected WS interface)
    All,
    // Local "unsafe" context and accounts access
    IpcContext,
    // APIs for Generic Pub-Sub
    PubSub,
    // Fixed list of APis
    List(HashSet<Api>),
}

impl Default for ApiSet {
    fn default() -> Self {
        ApiSet::UnsafeContext
    }
}

impl PartialEq for ApiSet {
    fn eq(&self, other: &Self) -> bool {
        self.list_apis() == other.list_apis()
    }
}

impl std::fmt::Display for ApiSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut apis: Vec<_> = self.list_apis().into_iter().collect();
        apis.sort();
        let mut iter = apis.iter();
        let first = iter.next();
        let display = iter.fold(
            first.map(ToString::to_string).unwrap_or_default(),
            |mut a, b| {
                a.push(',');
                a.push_str(&b.to_string());
                a
            },
        );
        write!(f, "{}", display)
    }
}

impl FromStr for ApiSet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut apis = HashSet::new();

        for api in s.split(',') {
            match api {
                "all" => {
                    apis.extend(ApiSet::All.list_apis());
                }
                "safe" => {
                    // Safe APIs are those that are safe even in UnsafeContext.
                    apis.extend(ApiSet::UnsafeContext.list_apis());
                }
                "ipc" => {
                    apis.extend(ApiSet::IpcContext.list_apis());
                }
                "pubsub" => {
                    apis.extend(ApiSet::PubSub.list_apis());
                }
                // Remove the API
                api if api.starts_with('-') => {
                    let api = api[1..].parse()?;
                    apis.remove(&api);
                }
                api => {
                    let api = api.parse()?;
                    apis.insert(api);
                }
            }
        }

        Ok(ApiSet::List(apis))
    }
}

impl Serialize for ApiSet {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ApiSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        ApiSet::from_str(&s).map_err(D::Error::custom)
    }
}

impl ApiSet {
    pub fn list_apis(&self) -> HashSet<Api> {
        let mut public_list: HashSet<Api> = [
            Api::Chain,
            Api::Miner,
            Api::Node,
            Api::State,
            Api::TxPool,
            Api::Contract,
        ]
        .iter()
        .cloned()
        .collect();

        match *self {
            ApiSet::UnsafeContext => public_list,
            ApiSet::List(ref apis) => apis.iter().cloned().collect(),

            ApiSet::IpcContext | ApiSet::All => {
                public_list.insert(Api::PubSub);
                public_list.insert(Api::Debug);
                public_list.insert(Api::Account);
                public_list.insert(Api::NetworkManager);
                public_list.insert(Api::SyncManager);
                public_list.insert(Api::NodeManager);
                public_list
            }

            ApiSet::PubSub => {
                public_list.insert(Api::PubSub);
                public_list
            }
        }
    }

    pub fn check_rpc_method(&self, method: &str) -> bool {
        let temp: Vec<&str> = method.split('.').collect();
        if temp.len() != 2 {
            return false;
        }
        Api::from_str(temp[0])
            .map(|api| self.list_apis().contains(&api))
            .unwrap_or(false)
    }
}
