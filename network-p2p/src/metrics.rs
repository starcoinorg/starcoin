use prometheus::Registry;
use starcoin_metrics::{
    register, HistogramOpts, HistogramVec, IntCounter, Opts, PrometheusError, UIntCounter,
    UIntCounterVec, UIntGauge, UIntGaugeVec,
};

#[derive(Clone)]
pub struct Metrics {
    // This list is ordered alphabetically
    pub connections_closed_total: UIntCounterVec,
    pub connections_opened_total: UIntCounterVec,
    pub distinct_peers_connections_closed_total: IntCounter,
    pub distinct_peers_connections_opened_total: IntCounter,
    pub incoming_connections_errors_total: UIntCounterVec,
    pub incoming_connections_total: IntCounter,
    pub kademlia_query_duration: HistogramVec,
    pub kademlia_random_queries_total: UIntCounterVec,
    pub kademlia_records_count: UIntGaugeVec,
    pub kademlia_records_sizes_total: UIntGaugeVec,
    pub kbuckets_num_nodes: UIntGaugeVec,
    pub listeners_local_addresses: UIntGauge,
    pub listeners_errors_total: IntCounter,
    pub notifications_sizes: HistogramVec,
    pub notifications_streams_closed_total: UIntCounter,
    pub notifications_streams_opened_total: UIntCounter,
    pub peerset_num_discovered: UIntGauge,
    pub peerset_num_requested: UIntGauge,
    pub pending_connections: UIntGauge,
    pub pending_connections_errors_total: UIntCounterVec,
    pub requests_in_failure_total: UIntCounterVec,
    pub requests_in_success_total: HistogramVec,
    pub requests_out_failure_total: UIntCounterVec,
    pub requests_out_success_total: HistogramVec,
    pub requests_out_started_total: UIntCounterVec,
    pub peerset_nodes: UIntGaugeVec,
}

impl Metrics {
    pub fn register(registry: &Registry) -> Result<Self, PrometheusError> {
        Ok(Self {
            // This list is ordered alphabetically
            connections_closed_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_connections_closed_total",
                        "Total number of connections closed, by direction and reason",
                    ),
                    &["direction", "reason"],
                )?,
                registry,
            )?,
            connections_opened_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_connections_opened_total",
                        "Total number of connections opened by direction",
                    ),
                    &["direction"],
                )?,
                registry,
            )?,
            distinct_peers_connections_closed_total: register(
                IntCounter::new(
                    "networkp2p_distinct_peers_connections_closed_total",
                    "Total number of connections closed with distinct peers",
                )?,
                registry,
            )?,
            distinct_peers_connections_opened_total: register(
                IntCounter::new(
                    "networkp2p_distinct_peers_connections_opened_total",
                    "Total number of connections opened with distinct peers",
                )?,
                registry,
            )?,
            incoming_connections_errors_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_incoming_connections_handshake_errors_total",
                        "Total number of incoming connections that have failed during the \
					initial handshake",
                    ),
                    &["reason"],
                )?,
                registry,
            )?,
            incoming_connections_total: register(
                IntCounter::new(
                    "networkp2p_incoming_connections_total",
                    "Total number of incoming connections on the listening sockets",
                )?,
                registry,
            )?,
            kademlia_query_duration: register(
                HistogramVec::new(
                    HistogramOpts {
                        common_opts: Opts::new(
                            "networkp2p_kademlia_query_duration",
                            "Duration of Kademlia queries per query type",
                        ),
                        buckets: prometheus::exponential_buckets(0.5, 2.0, 10)
                            .expect("parameters are always valid values; qed"),
                    },
                    &["type"],
                )?,
                registry,
            )?,
            kademlia_random_queries_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_kademlia_random_queries_total",
                        "Number of random Kademlia queries started",
                    ),
                    &["protocol"],
                )?,
                registry,
            )?,
            kademlia_records_count: register(
                UIntGaugeVec::new(
                    Opts::new(
                        "networkp2p_kademlia_records_count",
                        "Number of records in the Kademlia records store",
                    ),
                    &["protocol"],
                )?,
                registry,
            )?,
            kademlia_records_sizes_total: register(
                UIntGaugeVec::new(
                    Opts::new(
                        "networkp2p_kademlia_records_sizes_total",
                        "Total size of all the records in the Kademlia records store",
                    ),
                    &["protocol"],
                )?,
                registry,
            )?,
            kbuckets_num_nodes: register(
                UIntGaugeVec::new(
                    Opts::new(
                        "networkp2p_kbuckets_num_nodes",
                        "Number of nodes per kbucket per Kademlia instance",
                    ),
                    &["protocol", "lower_ilog2_bucket_bound"],
                )?,
                registry,
            )?,
            listeners_local_addresses: register(
                UIntGauge::new(
                    "networkp2p_listeners_local_addresses",
                    "Number of local addresses we're listening on",
                )?,
                registry,
            )?,
            listeners_errors_total: register(
                IntCounter::new(
                    "networkp2p_listeners_errors_total",
                    "Total number of non-fatal errors reported by a listener",
                )?,
                registry,
            )?,
            notifications_sizes: register(
                HistogramVec::new(
                    HistogramOpts {
                        common_opts: Opts::new(
                            "networkp2p_notifications_sizes",
                            "Sizes of the notifications send to and received from all nodes",
                        ),
                        buckets: prometheus::exponential_buckets(64.0, 4.0, 8)
                            .expect("parameters are always valid values; qed"),
                    },
                    &["direction", "protocol"],
                )?,
                registry,
            )?,
            notifications_streams_closed_total: register(
                UIntCounter::new(
                    "networkp2p_notifications_streams_closed_total",
                    "Total number of notification substreams that have been closed",
                )?,
                registry,
            )?,
            notifications_streams_opened_total: register(
                UIntCounter::new(
                    "networkp2p_notifications_streams_opened_total",
                    "Total number of notification substreams that have been opened",
                )?,
                registry,
            )?,
            peerset_num_discovered: register(
                UIntGauge::new(
                    "networkp2p_peerset_num_discovered",
                    "Number of nodes stored in the peerset manager",
                )?,
                registry,
            )?,
            peerset_num_requested: register(
                UIntGauge::new(
                    "networkp2p_peerset_num_requested",
                    "Number of nodes that the peerset manager wants us to be connected to",
                )?,
                registry,
            )?,
            pending_connections: register(
                UIntGauge::new(
                    "networkp2p_pending_connections",
                    "Number of connections in the process of being established",
                )?,
                registry,
            )?,
            pending_connections_errors_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_pending_connections_errors_total",
                        "Total number of pending connection errors",
                    ),
                    &["reason"],
                )?,
                registry,
            )?,
            requests_in_failure_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_requests_in_failure_total",
                        "Total number of incoming requests that the node has failed to answer",
                    ),
                    &["protocol", "reason"],
                )?,
                registry,
            )?,
            requests_in_success_total: register(
                HistogramVec::new(
                    HistogramOpts {
                        common_opts: Opts::new(
                            "networkp2p_requests_in_success_total",
                            "Total number of requests received and answered",
                        ),
                        buckets: prometheus::exponential_buckets(0.001, 2.0, 16)
                            .expect("parameters are always valid values; qed"),
                    },
                    &["protocol"],
                )?,
                registry,
            )?,
            requests_out_failure_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_requests_out_failure_total",
                        "Total number of requests that have failed",
                    ),
                    &["protocol", "reason"],
                )?,
                registry,
            )?,
            requests_out_success_total: register(
                HistogramVec::new(
                    HistogramOpts {
                        common_opts: Opts::new(
                            "networkp2p_requests_out_success_total",
                            "For successful requests, time between a request's start and finish",
                        ),
                        buckets: prometheus::exponential_buckets(0.001, 2.0, 16)
                            .expect("parameters are always valid values; qed"),
                    },
                    &["protocol"],
                )?,
                registry,
            )?,
            requests_out_started_total: register(
                UIntCounterVec::new(
                    Opts::new(
                        "networkp2p_requests_out_started_total",
                        "Total number of requests emitted",
                    ),
                    &["protocol"],
                )?,
                registry,
            )?,
            peerset_nodes: register(
                UIntGaugeVec::new(
                    Opts::new("networkp2p_peerset_nodes", "nodes numbers in each peer set"),
                    &["sets"],
                )?,
                registry,
            )?,
        })
    }
}
