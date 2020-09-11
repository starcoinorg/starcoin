use actix::dev::ToEnvelope;
use actix::{Actor, Addr, Handler, Message};

pub trait SendSyncEventHandler<M>: Send {
    fn send_event(&self, event: M);
}

pub trait SendSyncMsgEventHandler<M: Message<Result = ()> + Send> {
    fn send_msg_event(&self, event: M);
}

pub trait CloneSyncEventHandler<M> {
    fn clone_handler(&self) -> Box<dyn SendSyncEventHandler<M>>;
}

// pub trait SyncEventHandler<M>: SendSyncEventHandler<M> + CloneSyncEventHandler<M> {}

impl<A, M: Message<Result = ()> + Send> SendSyncMsgEventHandler<M> for Addr<A>
where
    A::Context: ToEnvelope<A, M>,
    A: Actor + Handler<M>,
{
    fn send_msg_event(&self, event: M) {
        self.do_send(event)
    }
}

impl<A, M: Message<Result = ()> + Send> CloneSyncEventHandler<M> for Addr<A>
where
    A::Context: ToEnvelope<A, M>,
    A: Actor + Handler<M>,
{
    fn clone_handler(&self) -> Box<dyn SendSyncEventHandler<M>> {
        Box::new(self.clone())
    }
}

// impl<A, M: Message<Result = ()> + Send> SyncEventHandler<M> for Addr<A> where
//     A::Context: ToEnvelope<A, M>,
//     A: Actor + Handler<M>{
// }

impl<A, M: Message<Result = ()> + Send> SendSyncEventHandler<M> for Addr<A>
where
    A::Context: ToEnvelope<A, M>,
    A: Actor + Handler<M>,
{
    fn send_event(&self, event: M) {
        self.send_msg_event(event);
    }
}
