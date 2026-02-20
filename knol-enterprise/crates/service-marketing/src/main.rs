//! Marketing service — autonomous social channel publishing with rate limiting.
//!
//! Port: 8088
//! Pattern: Axum HTTP routes + tokio-cron-scheduler for campaigns.
#![allow(dead_code)]

mod channels;
mod config;
mod config_loader;
mod content;
mod error;
mod rate_limiter;
mod routes;
mod scheduler;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::scheduler::campaigns;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "service_marketing=info,tower_http=info".into()),
        )
        .init();

    info!("Starting marketing-service (zero-cost strategy)");

    // Build application state
    let state = AppState::from_env().await?;

    // Start campaign scheduler
    let scheduler = start_scheduler(state.clone()).await?;

    // Build Axum router
    let app = Router::new()
        .route("/health", get(routes::health::health))
        .route("/internal/marketing/status", get(routes::status::status))
        .route(
            "/internal/marketing/rate-limits",
            get(routes::status::rate_limits),
        )
        .route("/internal/marketing/history", get(routes::status::history))
        .route(
            "/internal/marketing/trigger/{name}",
            post(routes::trigger::trigger),
        )
        .route("/internal/marketing/preview", get(routes::preview::preview))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8088));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Run server — scheduler runs in background
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(scheduler))
        .await?;

    Ok(())
}

async fn start_scheduler(state: Arc<AppState>) -> anyhow::Result<JobScheduler> {
    let sched = JobScheduler::new().await?;

    for campaign in campaigns::all_campaigns() {
        if !campaign.enabled {
            continue;
        }

        let state = state.clone();
        let name = campaign.name.clone();
        let cron = campaign.cron.clone();

        let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
            let state = state.clone();
            let name = name.clone();
            Box::pin(async move {
                info!("Scheduler: running campaign '{}'", name);
                match campaigns::execute_campaign(&state, &name, false).await {
                    Ok(results) => {
                        let successes = results.iter().filter(|r| r.status == "success").count();
                        info!(
                            "Scheduler: campaign '{}' completed — {}/{} succeeded",
                            name,
                            successes,
                            results.len()
                        );
                    }
                    Err(e) => {
                        error!("Scheduler: campaign '{}' failed — {}", name, e);
                    }
                }
            })
        })?;

        sched.add(job).await?;
        info!(
            "Scheduled campaign '{}' with cron '{}'",
            campaign.name, campaign.cron
        );
    }

    sched.start().await?;
    info!("Campaign scheduler started");

    Ok(sched)
}

async fn shutdown_signal(mut scheduler: JobScheduler) {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    info!("Shutting down marketing-service");
    let _ = scheduler.shutdown().await;
}
