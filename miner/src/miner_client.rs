use crate::miner::{MineCtx, Miner};
use crate::stratum::StratumManager;
use argon2::{self, Config};
use bus::BusActor;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use jsonrpc_core::futures::{future, stream::Stream, Future};
use jsonrpc_core::{request, IoHandler};
use jsonrpc_tcp_server::tokio::{
    io,
    io::lines,
    net::TcpStream,
    runtime,
    runtime::Runtime,
    timer::{timeout, Timeout},
};
use rand::Rng;
use sc_stratum::PushWorkHandler;
use sc_stratum::Stratum;
use std::io::{self as stdio, SeekFrom};
use std::net::Shutdown;
use std::sync::Arc;
use types::block::{Block, BlockHeader, BlockTemplate};
use types::{H256, U256};

pub fn calculate_hash(header: &[u8]) -> H256 {
    let config = Config::default();
    let output = argon2::hash_raw(header, header, &config).unwrap();
    let h_256: H256 = output.as_slice().into();
    h_256
}

pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 8);
    let _ = header.write_u64::<LittleEndian>(nonce);
    header
}

pub fn slove(difficulty: U256, header: &[u8]) -> u64 {
    let mut nonce = generate_nonce();
    loop {
        let pow_hash = calculate_hash(&set_header_nonce(header, nonce));
        let hash_u256: U256 = pow_hash.into();
        if hash_u256 > difficulty {
            nonce += 1;
            continue;
        }
        break;
    }
    nonce
}

pub fn verify(header: &[u8], nonce: u64, difficulty: U256) -> bool {
    let pow_header = set_header_nonce(header, nonce);
    let pow_hash = calculate_hash(&pow_header);
    let hash_u256: U256 = pow_hash.into();
    if hash_u256 <= difficulty {
        return true;
    }
    return false;
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}

fn mint() {
    let _system = actix::System::new("test");
    let mut miner = Miner::new(BusActor::launch());
    let stratum = {
        let addr = "127.0.0.1:9000".parse().unwrap();
        let dispatcher = Arc::new(StratumManager::new(miner.clone()));
        Stratum::start(&addr, dispatcher, None).unwrap()
    };
    let mine_ctx = {
        let block = Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test());
        let block_template = BlockTemplate::from_block(block);
        MineCtx::new(block_template)
    };

    miner.set_mint_job(mine_ctx);

    let addr = "127.0.0.1:9000".parse().unwrap();
    let mut runtime = Runtime::new().unwrap();
    let mut auth_request =
        r#"{"jsonrpc": "2.0", "method": "mining.authorize", "params": ["miner1", ""], "id": 1}"#
            .as_bytes()
            .to_vec();
    auth_request.extend(b"\n");
    let auth_response = "{\"jsonrpc\":\"2.0\",\"result\":true,\"id\":1}";
    let read_buf0 = Vec::<u8>::new();

    let stream = TcpStream::connect(&addr)
        .and_then(move |stream| io::write_all(stream, auth_request))
        .and_then(|(stream, _)| io::read_until(std::io::BufReader::new(stream), b'\n', read_buf0))
        .map_err(|err| panic!("{:?}", err))
        .and_then(move |(stream, read_buf0)| {
            // todo::process auth response
            println!("{:?}", String::from_utf8(read_buf0).unwrap());
            stratum.push_work_all(miner.get_mint_job()).unwrap();
            Timeout::new(
                future::ok(lines(stream)),
                ::std::time::Duration::from_millis(1000),
            )
        })
        .map_err(|err: timeout::Error<()>| panic!("Timeout: {:?}", err))
        .and_then(|lines| {
            lines.for_each(|line| {
                println!("{:?}", line);
                Ok(())
            })
        });
    runtime
        .block_on(stream)
        .expect("Runtime should run with no errors");
}

#[cfg(test)]
mod test {
    use crate::miner::{MineCtx, Miner};
    use crate::miner_client::{calculate_hash, mint, slove, verify};
    use crate::stratum::StratumManager;
    use actix_rt::Runtime;
    use bus::BusActor;
    use sc_stratum::PushWorkHandler;
    use sc_stratum::Stratum;
    use std::sync::Arc;
    use types::block::{Block, BlockHeader, BlockTemplate};

    fn prepare() {
        let _system = actix::System::new("test");
        let mut miner = Miner::new(BusActor::launch());
        let stratum = {
            let addr = "127.0.0.1:9000".parse().unwrap();
            let dispatcher = Arc::new(StratumManager::new(miner.clone()));
            Stratum::start(&addr, dispatcher, None).unwrap()
        };
        let mine_ctx = {
            let block = Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test());
            let block_template = BlockTemplate::from_block(block);
            MineCtx::new(block_template)
        };

        miner.set_mint_job(mine_ctx);
        stratum.push_work_all(miner.get_mint_job()).unwrap();
    }

    #[test]
    fn test_mine() {
        mint();
    }

    #[test]
    fn test_hash() {
        let header = "hellostarcoin".as_bytes();
        let df = types::U256::max_value() / 2.into();
        let nonce = slove(df, header.clone());
        let verified = verify(header, nonce, df);
        assert_eq!(true, verified);
    }
}
