use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use futures::{TryStream, TryStreamExt};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::TypeTagView;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::event::EventKey;
use tokio::io::AsyncBufReadExt;

/// Subscribe chain event.
#[derive(Debug, Parser)]
#[clap(name = "event")]
pub struct SubscribeEventOpt {
    #[clap(
        short = 'f',
        long = "from",
        name = "from_block",
        help = "from block number"
    )]
    from_block: Option<u64>,
    #[clap(short = 't', long = "to", name = "to_block", help = "to block number")]
    to_block: Option<u64>,
    #[arg(
        short = 'k',
        long = "event-key",
        name = "event_key",
        help = "event key"
    )]
    event_key: Option<Vec<EventKey>>,
    #[arg(long = "address", name = "address")]
    /// events of which addresses to subscribe
    addresses: Option<Vec<AccountAddress>>,
    #[arg(long = "type_tag", name = "type-tag")]
    /// type tags of the events to subscribe
    type_tags: Option<Vec<TypeTagView>>,
    #[clap(short = 'l', long = "limit", name = "limit")]
    /// limit return size
    limit: Option<usize>,
    #[clap(long = "decode")]
    /// whether decode event
    decode: bool,
}

pub struct SubscribeEventCommand;
impl CommandAction for SubscribeEventCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubscribeEventOpt;
    type ReturnItem = ();
    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let filter = EventFilter {
            from_block: ctx.opt().from_block,
            to_block: ctx.opt().to_block,
            event_keys: ctx.opt().event_key.clone(),
            addrs: ctx.opt().addresses.clone(),
            type_tags: ctx.opt().type_tags.clone(),
            limit: ctx.opt().limit,
        };

        let event_stream = ctx
            .state()
            .client()
            .subscribe_events(filter, ctx.opt().decode)?;
        println!("Subscribe successful, Press `q` and Enter to quit");
        blocking_display_notification(event_stream, |evt| {
            serde_json::to_string(&evt).expect("should never fail")
        });

        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(name = "new_block")]
pub struct SubscribeBlockOpt {}
pub struct SubscribeBlockCommand;
impl CommandAction for SubscribeBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubscribeBlockOpt;
    type ReturnItem = ();
    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let event_stream = ctx.state().client().subscribe_new_blocks()?;
        println!("Subscribe successful, Press `q` and Enter to quit");
        blocking_display_notification(event_stream, |evt| {
            serde_json::to_string(&evt).expect("should never fail")
        });
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(name = "new_mint_block")]
pub struct SubscribeNewMintBlockOpt {}
pub struct SubscribeNewMintBlockCommand;
impl CommandAction for SubscribeNewMintBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubscribeNewMintBlockOpt;
    type ReturnItem = ();
    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let event_stream = ctx.state().client().subscribe_new_mint_blocks()?;
        println!("Subscribe successful, Press `q` and Enter to quit");
        blocking_display_notification(event_stream, |evt| {
            serde_json::to_string(&evt).expect("should never fail")
        });
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(name = "new_pending_txn")]
pub struct SubscribeNewTxnOpt {}
pub struct SubscribeNewTxnCommand;
impl CommandAction for SubscribeNewTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = SubscribeNewTxnOpt;
    type ReturnItem = ();
    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let event_stream = ctx.state().client().subscribe_new_transactions()?;
        println!("Subscribe successful, Press `q` and Enter to quit");
        blocking_display_notification(event_stream, |evt| {
            serde_json::to_string(&evt).expect("should never fail")
        });
        Ok(())
    }
}

fn blocking_display_notification<T, F>(
    mut event_stream: impl TryStream<Ok = T, Error = anyhow::Error> + Unpin,
    display: F,
) where
    F: Fn(&T) -> String,
{
    let rt = tokio::runtime::Builder::new_multi_thread()
        .build()
        .expect("should able to create tokio runtime");
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    rt.block_on(async move {
        loop {
            tokio::select! {
               maybe_quit = lines.next_line()  => {
                   if let Ok(Some(q)) = maybe_quit {
                       if q.as_str() == "q" {
                           break;
                       }
                   }
               }
               try_event = event_stream.try_next() => {
                   match try_event {
                        Ok(None) => break,
                        Ok(Some(evt)) => {
                            println!("{}", display(&evt));
                        }
                        Err(e) => {
                            eprintln!("subscription return err: {}", &e);
                        }
                   }
               }
            }
        }
    });
}
