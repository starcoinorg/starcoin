use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use futures::{StreamExt, TryStream, TryStreamExt};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_types::event::EventKey;
use std::convert::TryFrom;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, StructOpt)]
#[structopt(name = "event")]
pub struct SubscribeEventOpt {
    #[structopt(
        short = "f",
        long = "from",
        name = "from_block",
        help = "from block number"
    )]
    from_block: Option<u64>,
    #[structopt(short = "t", long = "to", name = "to_block", help = "to block number")]
    to_block: Option<u64>,
    #[structopt(
        short = "k",
        long = "event-key",
        name = "event_key",
        help = "event key",
        multiple = true,
        parse(try_from_str=parse_event_key)
    )]
    event_key: Vec<EventKey>,
    #[structopt(
        short = "l",
        long = "limit",
        name = "limit",
        help = "limit return size"
    )]
    limit: Option<usize>,
}

fn parse_event_key(s: &str) -> Result<EventKey> {
    let b = hex::decode(s)?;
    EventKey::try_from(b.as_slice())
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
            limit: ctx.opt().limit,
        };

        let event_stream = ctx.state().client().subscribe_events(filter)?;
        println!("Subscribe successful, Press `q` and Enter to quit");
        blocking_display_notification(event_stream, |evt| {
            serde_json::to_string(&evt).expect("should never fail")
        });

        Ok(())
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "new_block")]
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
#[derive(Debug, StructOpt)]
#[structopt(name = "new_pending_txn")]
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
    let mut rt = tokio::runtime::Builder::new()
        .basic_scheduler()
        .build()
        .expect("should able to create tokio runtime");
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    rt.block_on(async move {
        loop {
            tokio::select! {
               maybe_quit = lines.next()  => {
                   if let Some(Ok(q)) = maybe_quit {
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
