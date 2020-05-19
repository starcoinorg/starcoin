use std::fmt;

use actix::dev::channel::AddressReceiver;
use actix::dev::{
    channel, AsyncContextParts, ContextFut, ContextParts, Envelope, Mailbox, ToEnvelope,
};
use actix::{
    Actor, ActorContext, ActorFuture, ActorState, Addr, Arbiter, AsyncContext, Handler, Message,
    SpawnHandle,
};
use futures::channel::oneshot::Sender;

/// An actor execution context.
pub struct Context<A>
where
    A: Actor<Context = Context<A>>,
{
    parts: ContextParts<A>,
    mb: Option<Mailbox<A>>,
}

impl<A: Actor<Context = Context<A>>> fmt::Debug for Context<A> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Context")
            .field("parts", &self.parts)
            .field("mb", &self.mb)
            .finish()
    }
}

impl<A> ActorContext for Context<A>
where
    A: Actor<Context = Self>,
{
    #[inline]
    fn stop(&mut self) {
        self.parts.stop()
    }
    #[inline]
    fn terminate(&mut self) {
        self.parts.terminate()
    }
    #[inline]
    fn state(&self) -> ActorState {
        self.parts.state()
    }
}

impl<A> AsyncContext<A> for Context<A>
where
    A: Actor<Context = Self>,
{
    #[inline]
    fn spawn<F>(&mut self, fut: F) -> SpawnHandle
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        self.parts.spawn(fut)
    }

    #[inline]
    fn wait<F>(&mut self, fut: F)
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        self.parts.wait(fut)
    }

    #[inline]
    fn waiting(&self) -> bool {
        self.parts.waiting()
    }

    #[inline]
    fn cancel_future(&mut self, handle: SpawnHandle) -> bool {
        self.parts.cancel_future(handle)
    }

    #[inline]
    fn address(&self) -> Addr<A> {
        self.parts.address()
    }
}

impl<A> Context<A>
where
    A: Actor<Context = Self>,
{
    #[inline]
    pub fn new() -> Self {
        let mb = Mailbox::default();
        Self {
            parts: ContextParts::new(mb.sender_producer()),
            mb: Some(mb),
        }
    }

    #[inline]
    pub fn with_receiver(rx: AddressReceiver<A>) -> Self {
        let mb = Mailbox::new(rx);
        Self {
            parts: ContextParts::new(mb.sender_producer()),
            mb: Some(mb),
        }
    }

    #[inline]
    pub fn run(self, act: A) -> Addr<A> {
        let fut = self.into_future(act);
        let addr = fut.address();
        actix_rt::spawn(fut);
        addr
    }

    pub fn into_future(mut self, act: A) -> ContextFut<A, Self> {
        let mb = self.mb.take().unwrap();
        ContextFut::new(self, act, mb)
    }

    /// Returns a handle to the running future.
    ///
    /// This is the handle returned by the `AsyncContext::spawn()`
    /// method.
    pub fn handle(&self) -> SpawnHandle {
        self.parts.curr_handle()
    }

    #[allow(clippy::needless_doctest_main)]
    /// Sets the mailbox capacity.
    ///
    /// The default mailbox capacity is 16 messages.
    /// #Examples
    /// ```
    /// # use actix::prelude::*;
    /// struct MyActor;
    /// impl Actor for MyActor {
    ///     type Context = Context<Self>;
    ///
    ///     fn started(&mut self, ctx: &mut Self::Context) {
    ///         ctx.set_mailbox_capacity(1);
    ///     }
    /// }
    ///
    /// # fn main() {
    /// # System::new("test");
    /// let addr = MyActor.start();
    /// # }
    /// ```
    pub fn set_mailbox_capacity(&mut self, cap: usize) {
        self.parts.set_mailbox_capacity(cap)
    }

    /// Returns whether any addresses are still connected.
    pub fn connected(&self) -> bool {
        self.parts.connected()
    }
}

impl<A> Default for Context<A>
where
    A: Actor<Context = Self>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> AsyncContextParts<A> for Context<A>
where
    A: Actor<Context = Self>,
{
    fn parts(&mut self) -> &mut ContextParts<A> {
        &mut self.parts
    }
}

impl<A, M> ToEnvelope<A, M> for Context<A>
where
    A: Actor<Context = Context<A>> + Handler<M>,
    M: Message + Send + 'static,
    M::Result: Send,
{
    fn pack(msg: M, tx: Option<Sender<M::Result>>) -> Envelope<A> {
        Envelope::new(msg, tx)
    }
}

pub fn start_in_arbiter<S, F>(arb: &Arbiter, f: F) -> Addr<S>
where
    S: Actor<Context = Context<S>>,
    F: FnOnce(&mut Context<S>) -> S + Send + 'static,
{
    let (tx, rx) = channel::channel(16);

    // create actor
    arb.exec_fn(move || {
        let mut ctx = Context::with_receiver(rx);
        let act = f(&mut ctx);
        let fut = ctx.into_future(act);

        actix_rt::spawn(fut);
    });

    Addr::new(tx)
}

mod test {
    use crate::Context;
    use actix::{Actor, Handler, Message};

    struct MyActor {
        count: usize,
    }

    impl Actor for MyActor {
        type Context = Context<Self>;
    }

    struct Ping(usize);

    impl Message for Ping {
        type Result = usize;
    }

    impl Handler<Ping> for MyActor {
        type Result = usize;

        fn handle(&mut self, msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
            self.count += msg.0;

            self.count
        }
    }

    #[test]
    fn test_context() {
        use crate::start_in_arbiter;
        use actix::{Arbiter, System};
        use futures::future::FutureExt;

        let system = System::new("test");
        let addr = start_in_arbiter(&Arbiter::current(), |_| MyActor { count: 10 });

        let res = addr.send(Ping(10));

        Arbiter::spawn(
            res.map(|res| {
                println!("RESULT: {}", res.unwrap() == 20);
                System::current().stop();
            }), //.instrument(tracing::span!(Level::TRACE, "bar")),
        );

        system.run().unwrap();
    }
}
