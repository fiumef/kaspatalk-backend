#![allow(warnings)]

use axum::{
    routing::get
    , Router,
};
use clap::Parser;
pub use futures::{select_biased, FutureExt, Stream, StreamExt, TryStreamExt};
use kaspa_notify::notification::test_helpers::Data;
use polodb_core::bson::doc;
use polodb_core::{Collection, CollectionT, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
// We use workflow-rs primitives for async task and channel management
// as they function uniformly in tokio as well as WASM32 runtimes.
use workflow_core::channel::oneshot;
use workflow_log::prelude::*;

// Kaspa RPC primitives
use kaspa_wrpc_client::prelude::*;
// reuse wRPC Result type for convenience
use kaspa_wrpc_client::result::Result;

mod message;
use message::*;
mod listener;
use listener::*;
mod api;
use api::*;
mod cli_parser;
mod inner;
mod query_parameters;


struct AppState {
    database: Database,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli_parser::Args::parse();
    // Building database
    let db = Database::open_path("kaspatalk.db").unwrap();
    let stored_messages: Collection<Message> = db.collection("stored-messages");
    // Building rusty-kaspa listener
    let rusty_kaspa_address_prefix = "ws://".to_owned();
    let rusty_kaspa_url = args.rusty_kaspa_address();
    let complete_rusty_kaspa_address = format!("{rusty_kaspa_address_prefix}{rusty_kaspa_url}");
    let listener = Listener::try_new(
        NetworkId::with_suffix(NetworkType::Testnet, 11),
        Some(complete_rusty_kaspa_address),
        stored_messages,
    )?;
    listener.start().await?;

    // Building webserver app
    let app_state = Arc::new(AppState { database: db });
    let router = Router::new()
        .route("/get-all-messages", get(get_all_messages))
        .route("/get-messages", get(api::get_messages))
        .route("/get-peers", get(api::get_peers))
        .with_state(app_state);

    // Serving webserver app
    let webserver = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(webserver, router).await.unwrap();

    // Shutdown procedure
    let (shutdown_sender, shutdown_receiver) = oneshot::<()>();
    ctrlc::set_handler(move || {
        log_info!("^SIGTERM - shutting down...");
        shutdown_sender
            .try_send(())
            .expect("Error sending shutdown signal...");
    })
    .expect("Unable to set the Ctrl+C signal handler");
    shutdown_receiver
        .recv()
        .await
        .expect("Error waiting for shutdown signal...");
    listener.stop().await?;
    Ok(())
}
