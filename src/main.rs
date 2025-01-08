mod config;

use chrono::{DateTime, Utc};
use clap::Parser;
use config::Config;
use prometheus::{register_int_gauge_vec, Encoder, IntGaugeVec, TextEncoder};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::interval;
use warp::Filter;

#[derive(Deserialize, Clone, Eq, Hash, PartialEq, Debug)]
struct ChainUpgrade {
    network: String,
    node_version: String,
    estimated_upgrade_time: String,
    block: u64,
}

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let config = Config::from_file(&opts.config);

    println!(
        "Watching the following chains: {:?}",
        config.chain.watch_list
    );

    let client = Client::new();
    let gauge = register_int_gauge_vec!(
        "chain_upgrade",
        "Chain upgrade information",
        &["network", "node_version", "block"]
    )
    .unwrap();

    let watch_list: Vec<String> = config.chain.watch_list.clone();
    let refresh_interval = humantime::parse_duration(&config.chain.refresh).unwrap();

    let upgrades: Arc<Mutex<HashSet<ChainUpgrade>>> = Arc::new(Mutex::new(HashSet::new()));

    let upgrades_clone = Arc::clone(&upgrades);
    let client_clone = client.clone();
    let gauge_clone = gauge.clone();
    let watch_list_clone = watch_list.clone();
    let endpoint_clone = config.chain.endpoint.clone();

    tokio::spawn(async move {
        let mut interval = interval(refresh_interval);
        loop {
            interval.tick().await;
            match fetch_and_update_metrics(
                &client_clone,
                &endpoint_clone,
                &watch_list_clone,
                &gauge_clone,
                &upgrades_clone,
            )
            .await
            {
                Ok(_) => println!("Metrics updated successfully"),
                Err(e) => eprintln!("Failed to update metrics: {}", e),
            }
        }
    });

    let metrics_route = warp::path("metrics").map(move || {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        warp::http::Response::builder()
            .header("Content-Type", encoder.format_type())
            .body(buffer)
    });

    warp::serve(metrics_route)
        .run((
            config.prometheus.host.parse::<IpAddr>().unwrap(),
            config.prometheus.port,
        ))
        .await;
}

async fn fetch_and_update_metrics(
    client: &Client,
    endpoint: &str,
    watch_list: &[String],
    gauge: &IntGaugeVec,
    upgrades: &Arc<Mutex<HashSet<ChainUpgrade>>>,
) -> Result<(), reqwest::Error> {
    let response = client
        .get(endpoint)
        .send()
        .await?
        .json::<Vec<ChainUpgrade>>()
        .await?;
    let now = Utc::now();
    let mut upgrades_guard = upgrades.lock().await;

    // Add or update metrics
    for upgrade in &response {
        if watch_list.contains(&upgrade.network) {
            let upgrade_time =
                DateTime::parse_from_rfc3339(&upgrade.estimated_upgrade_time).unwrap();
            if now < upgrade_time {
                gauge
                    .with_label_values(&[
                        &upgrade.network,
                        &upgrade.node_version,
                        &upgrade.block.to_string(),
                    ])
                    .set(1);
                upgrades_guard.insert(upgrade.clone());
            }
        }
    }

    // Remove expired metrics
    upgrades_guard.retain(|upgrade| {
        let upgrade_time = DateTime::parse_from_rfc3339(&upgrade.estimated_upgrade_time).unwrap();
        if now >= upgrade_time {
            let _ = gauge.remove_label_values(&[
                &upgrade.network,
                &upgrade.node_version,
                &upgrade.block.to_string(),
            ]);
            false
        } else {
            true
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_add_update_metrics() {
        let gauge = register_int_gauge_vec!(
            "test_chain_upgrade",
            "Test chain upgrade information",
            &["network", "node_version", "block"]
        )
        .unwrap();

        let upgrades: Arc<Mutex<HashSet<ChainUpgrade>>> = Arc::new(Mutex::new(HashSet::new()));
        let watch_list = vec!["testnet".to_string()];

        let now = Utc::now();
        let future_time = (now + Duration::hours(1)).to_rfc3339();
        let future_upgrade = "v1.1";
        let past_time = (now - Duration::hours(1)).to_rfc3339();
        let past_upgrade = "v1.0";

        let upgrade_future = ChainUpgrade {
            network: "testnet".to_string(),
            node_version: future_upgrade.to_string(),
            estimated_upgrade_time: future_time.clone(),
            block: 12345,
        };

        let upgrade_past = ChainUpgrade {
            network: "testnet".to_string(),
            node_version: past_upgrade.to_string(),
            estimated_upgrade_time: past_time.clone(),
            block: 12345,
        };

        let mut server = mockito::Server::new_async().await;

        server.mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"[{{"network":"testnet","node_version":"{}","estimated_upgrade_time":"{}","block":12345}}]"#,
                future_upgrade,
                future_time
            ))
            .create();

        {
            let mut upgrades_guard = upgrades.lock().await;
            upgrades_guard.insert(upgrade_future.clone());
            upgrades_guard.insert(upgrade_past.clone());
        }

        fetch_and_update_metrics(
            &Client::new(),
            &server.url(),
            &watch_list,
            &gauge,
            &upgrades,
        )
        .await
        .unwrap();

        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains(future_upgrade));
        assert!(!output.contains(past_upgrade));
    }
}
