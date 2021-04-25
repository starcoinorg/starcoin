// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct MetricsActorService {
    push_url: String,
    auth_username: Option<String>,
    auth_password: String,
    interval: u64,
    push_status: Arc<AtomicBool>,
}

impl MetricsActorService {
    pub fn new(
        push_url: String,
        interval: u64,
        auth_username: Option<String>,
        auth_password: String,
        status: bool,
    ) -> Self {
        Self {
            push_url,
            interval,
            auth_username,
            auth_password,
            push_status: Arc::new(AtomicBool::new(status)),
        }
    }
}
impl ServiceFactory<Self> for MetricsActorService {
    fn create(ctx: &mut ServiceContext<MetricsActorService>) -> Result<MetricsActorService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        Ok(Self::new(
            config.metrics.push_config.push_server_url(),
            config.metrics.push_config.interval(),
            config.metrics.push_config.auth_username(),
            config.metrics.push_config.auth_password(),
            !config.metrics.disable_metrics() && config.metrics.push_config.is_config(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct PushMetricsEvent {
    push_url: String,
    auth_username: Option<String>,
    auth_password: String,
}
impl PushMetricsEvent {
    pub fn new(push_url: String, auth_username: Option<String>, auth_password: String) -> Self {
        Self {
            push_url,
            auth_username,
            auth_password,
        }
    }
}

impl ActorService for MetricsActorService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if self.push_status.load(Ordering::Relaxed) {
            ctx.subscribe::<PushMetricsEvent>();
            let push_url = self.push_url.clone();
            let auth_username = self.auth_username.clone();
            let auth_password = self.auth_password.clone();
            ctx.run_interval(Duration::from_secs(self.interval), move |ctx| {
                ctx.broadcast(PushMetricsEvent::new(
                    push_url.clone(),
                    auth_username.clone(),
                    auth_password.clone(),
                ));
            });
        }
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if self.push_status.load(Ordering::Relaxed) {
            ctx.unsubscribe::<PushMetricsEvent>();
        }
        Ok(())
    }
}

impl EventHandler<Self, PushMetricsEvent> for MetricsActorService {
    fn handle_event(&mut self, msg: PushMetricsEvent, _ctx: &mut ServiceContext<Self>) {
        starcoin_metrics::metric_server::push_metrics(
            msg.push_url,
            msg.auth_username,
            msg.auth_password,
        );
    }
}
