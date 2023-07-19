mod certification;
mod chat_gpt_api;
mod creature;
mod error_conversion;
mod rpc_context;

use crate::chat_gpt_api::memory::FiniteQueueMemory;
use crate::chat_gpt_api::specification::Model;
use crate::creature::my_creature::creature_rpc::creature_server::CreatureServer;
use crate::creature::my_creature::MyCreature;
use crate::rpc_context::RpcContext;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let span = tracing::span!(tracing::Level::DEBUG, "main");
    let _enter = span.enter();

    tracing::info!("Starting server...");

    let address = "0.0.0.0:8000".parse().map_err(|error| {
        tracing::error!("Failed to parse address: {:?}", error);
        error
    })?;

    // create our state
    let model = Model::Gpt35Turbo0613;
    let prompt = "Your are an AI assistant.".to_string();
    let context_memory = FiniteQueueMemory::new(10);
    let rpc_context = Arc::new(Mutex::new(RpcContext {
        model,
        prompt,
        context_memory,
    }));

    let creature = MyCreature {
        context: rpc_context,
    };

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            crate::creature::my_creature::creature_rpc::FILE_DESCRIPTOR_SET,
        )
        .build()
        .map_err(|error| {
            tracing::error!("Failed to create reflection server: {:?}", error);
            error
        })?;

    Server::builder()
        .tls_config(crate::certification::build_tls_config().map_err(|error| {
            tracing::error!("Failed to build TLS config: {:?}", error);
            error
        })?)
        .map_err(|error| {
            tracing::error!("Failed to create server: {:?}", error);
            error
        })?
        .add_service(CreatureServer::new(creature))
        .add_service(reflection_server)
        .serve(address)
        .await
        .map_err(|error| {
            tracing::error!("Failed to serve: {:?}", error);
            error
        })?;

    tracing::info!("Server is running on {}", address);

    Ok(())
}
