// This file is part of Substrate.
//
// Copyright (C) 2018-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use futures::{
    future::{MapOk, TryFutureExt},
    io::{IoSlice, IoSliceMut},
    prelude::*,
    ready,
};
use libp2p::{
    core::{
        muxing::{StreamMuxer, StreamMuxerBox, StreamMuxerEvent},
        transport::{Boxed, DialOpts, ListenerId, MemoryTransport, TransportError, TransportEvent},
        upgrade,
    },
    dns, identity, noise, tcp, PeerId, Transport,
};
use std::{
    convert::TryFrom as _,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};

#[derive(Debug, Default)]
pub struct BandwidthSinks {
    inbound: AtomicU64,
    outbound: AtomicU64,
}

impl BandwidthSinks {
    pub fn total_inbound(&self) -> u64 {
        self.inbound.load(Ordering::Relaxed)
    }

    pub fn total_outbound(&self) -> u64 {
        self.outbound.load(Ordering::Relaxed)
    }

    fn record_inbound(&self, bytes: usize) {
        self.inbound
            .fetch_add(u64::try_from(bytes).unwrap_or(u64::MAX), Ordering::Relaxed);
    }

    fn record_outbound(&self, bytes: usize) {
        self.outbound
            .fetch_add(u64::try_from(bytes).unwrap_or(u64::MAX), Ordering::Relaxed);
    }
}

#[derive(Clone)]
#[pin_project::pin_project]
struct BandwidthTransport<T> {
    #[pin]
    transport: T,
    sinks: Arc<BandwidthSinks>,
}

impl<T> BandwidthTransport<T> {
    fn new(transport: T) -> (Self, Arc<BandwidthSinks>) {
        let sinks = Arc::new(BandwidthSinks::default());
        (
            Self {
                transport,
                sinks: sinks.clone(),
            },
            sinks,
        )
    }
}

impl<T, M> Transport for BandwidthTransport<T>
where
    T: Transport<Output = (PeerId, M)>,
    M: StreamMuxer + Send + 'static,
    M::Substream: Send + 'static,
    M::Error: Send + Sync + 'static,
{
    type Output = (PeerId, BandwidthMuxer<M>);
    type Error = T::Error;
    type ListenerUpgrade = MapOk<
        T::ListenerUpgrade,
        Box<dyn FnOnce((PeerId, M)) -> (PeerId, BandwidthMuxer<M>) + Send>,
    >;
    type Dial = MapOk<T::Dial, Box<dyn FnOnce((PeerId, M)) -> (PeerId, BandwidthMuxer<M>) + Send>>;

    fn listen_on(
        &mut self,
        id: ListenerId,
        addr: libp2p::Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        self.transport.listen_on(id, addr)
    }

    fn remove_listener(&mut self, id: ListenerId) -> bool {
        self.transport.remove_listener(id)
    }

    fn dial(
        &mut self,
        addr: libp2p::Multiaddr,
        dial_opts: DialOpts,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        let sinks = self.sinks.clone();
        Ok(self.transport.dial(addr, dial_opts)?.map_ok(Box::new(
            move |(peer_id, stream_muxer)| {
                (
                    peer_id,
                    BandwidthMuxer {
                        inner: stream_muxer,
                        sinks,
                    },
                )
            },
        )))
    }

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        let this = self.project();
        match this.transport.poll(cx) {
            Poll::Ready(TransportEvent::Incoming {
                listener_id,
                upgrade,
                local_addr,
                send_back_addr,
            }) => {
                let sinks = this.sinks.clone();
                Poll::Ready(TransportEvent::Incoming {
                    listener_id,
                    upgrade: upgrade.map_ok(Box::new(move |(peer_id, stream_muxer)| {
                        (
                            peer_id,
                            BandwidthMuxer {
                                inner: stream_muxer,
                                sinks,
                            },
                        )
                    })),
                    local_addr,
                    send_back_addr,
                })
            }
            Poll::Ready(other) => {
                let mapped = other.map_upgrade(|_upgrade| unreachable!("case already matched"));
                Poll::Ready(mapped)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone)]
#[pin_project::pin_project]
struct BandwidthMuxer<M> {
    #[pin]
    inner: M,
    sinks: Arc<BandwidthSinks>,
}

impl<M> StreamMuxer for BandwidthMuxer<M>
where
    M: StreamMuxer,
{
    type Substream = BandwidthStream<M::Substream>;
    type Error = M::Error;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<StreamMuxerEvent, Self::Error>> {
        self.project().inner.poll(cx)
    }

    fn poll_inbound(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Substream, Self::Error>> {
        let this = self.project();
        let inner = ready!(this.inner.poll_inbound(cx)?);
        Poll::Ready(Ok(BandwidthStream {
            inner,
            sinks: this.sinks.clone(),
        }))
    }

    fn poll_outbound(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Self::Substream, Self::Error>> {
        let this = self.project();
        let inner = ready!(this.inner.poll_outbound(cx)?);
        Poll::Ready(Ok(BandwidthStream {
            inner,
            sinks: this.sinks.clone(),
        }))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

#[pin_project::pin_project]
struct BandwidthStream<S> {
    #[pin]
    inner: S,
    sinks: Arc<BandwidthSinks>,
}

impl<S: AsyncRead> AsyncRead for BandwidthStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_read(cx, buf))?;
        this.sinks.record_inbound(num_bytes);
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_read_vectored(cx, bufs))?;
        this.sinks.record_inbound(num_bytes);
        Poll::Ready(Ok(num_bytes))
    }
}

impl<S: AsyncWrite> AsyncWrite for BandwidthStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_write(cx, buf))?;
        this.sinks.record_outbound(num_bytes);
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let num_bytes = ready!(this.inner.poll_write_vectored(cx, bufs))?;
        this.sinks.record_outbound(num_bytes);
        Poll::Ready(Ok(num_bytes))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_close(cx)
    }
}

/// Builds the base transport used by `network-p2p`.
pub fn build_transport(
    keypair: identity::Keypair,
    memory_only: bool,
) -> (Boxed<(PeerId, StreamMuxerBox)>, Arc<BandwidthSinks>) {
    let noise_config = noise::Config::new(&keypair)
        .expect("Noise static DH key generation from identity keypair must succeed");
    let yamux_config = libp2p::yamux::Config::default();

    let transport = if memory_only {
        MemoryTransport::default()
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise_config)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed()
    } else {
        let tcp_config = tcp::Config::new().nodelay(true);
        let tcp_transport = tcp::tokio::Transport::new(tcp_config);
        let base = match dns::tokio::Transport::system(tcp_transport) {
            Ok(dns_transport) => dns_transport.boxed(),
            Err(_) => tcp::tokio::Transport::new(tcp::Config::new().nodelay(true)).boxed(),
        };

        base.upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&keypair).expect("noise config must succeed"))
            .multiplex(libp2p::yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed()
    };

    let (transport, bandwidth) = BandwidthTransport::new(transport);
    let transport = transport
        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)))
        .boxed();

    (transport, bandwidth)
}
