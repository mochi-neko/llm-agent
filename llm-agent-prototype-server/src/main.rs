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
    let address = "0.0.0.0:8000".parse()?;

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
        .build()?;

    Server::builder()
        .tls_config(crate::certification::build_tls_config()?)?
        .add_service(CreatureServer::new(creature))
        .add_service(reflection_server)
        .serve(address)
        .await?;

    Ok(())
}
