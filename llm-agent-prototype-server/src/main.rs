mod certification;
mod chat_gpt_api;
mod creature;
mod error_mapping;
mod logging;
mod rpc_context;
mod vector_db;

use crate::chat_gpt_api::memory::FiniteQueueMemory;
use crate::chat_gpt_api::specification::Model;
use crate::creature::my_creature::creature_rpc::creature_server::CreatureServer;
use crate::creature::my_creature::MyCreature;
use crate::rpc_context::RpcContext;
use crate::vector_db::embeddings;
use qdrant_client::prelude::QdrantClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use vector_db::database::DataBase;

#[tracing::instrument(name = "main", err)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    crate::logging::initialize_logging().map_err(|error| {
        tracing::error!(
            "Failed to initialize logging: {:?}",
            error
        );
        error
    })?;

    tracing::info!("Starting server...");

    let address = "0.0.0.0:50051"
        .parse()
        .map_err(|error| {
            tracing::error!("Failed to parse address: {:?}", error);
            error
        })?;

    // create our state
    let model = Model::Gpt35Turbo0613;
    let prompt = "Your are an AI assistant.".to_string();
    let context_memory = FiniteQueueMemory::new(10);
    let qdrant_client = QdrantClient::from_url("http://qdrant:6334")
        .build()
        .map_err(|error| {
            tracing::error!(
                "Failed to create qdrant client: {:?}",
                error
            );
            error
        })?;
    qdrant_client
        .health_check()
        .await
        .map_err(|error| {
            tracing::error!(
                "Failed to check qdrant health: {:?}",
                error
            );
            error
        })?;
    let dimension = embeddings::get_dimension()
        .await
        .map_err(|error| {
            tracing::error!("Failed to get dimension: {:?}", error);
            error
        })?;
    let long_memory = DataBase::new(
        qdrant_client,
        "long_memory".to_string(),
        dimension,
    )
    .await
    .map_err(|error| {
        tracing::error!(
            "Failed to create long memory: {:?}",
            error
        );
        error
    })?;
    let rpc_context = Arc::new(Mutex::new(RpcContext {
        model,
        prompt,
        context_memory,
        long_memory,
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
            tracing::error!(
                "Failed to create reflection server: {:?}",
                error
            );
            error
        })?;

    tracing::info!("Server is running on {}", address);

    Server::builder()
        .tls_config(
            crate::certification::build_tls_config().map_err(|error| {
                tracing::error!(
                    "Failed to build TLS config: {:?}",
                    error
                );
                error
            })?,
        )
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

    Ok(())
}
