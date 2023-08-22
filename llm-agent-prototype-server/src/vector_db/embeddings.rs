use anyhow::{Error, Result};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use tokio::task;

#[tracing::instrument(name = "vector_db.embeddings.embed", err)]
pub(crate) async fn embed(sentence: String) -> Result<Vec<Vec<f32>>> {
    let embeddings_result: Result<Vec<Vec<f32>>, Error> =
        task::spawn_blocking(move || {
            // Setup sentence embeddings model
            let model = SentenceEmbeddingsBuilder::remote(
                SentenceEmbeddingsModelType::AllMiniLmL6V2,
            )
            .create_model()
            .map_err(|error| {
                tracing::error!("Failed to create model: {:?}", error);
                error
            })?;

            // Generate Embeddings
            let embeddings = model
                .encode(&[sentence])
                .map_err(|error| {
                    tracing::error!("Failed to encode sentence: {:?}", error);
                    error
                })?;

            Ok(embeddings)
        })
        .await
        .map_err(|error| {
            tracing::error!(
                "Failed to spawn blocking task: {:?}",
                error
            );
            error
        })?;

    embeddings_result
}

#[tracing::instrument(
    name = "vector_db.embeddings.get_dimension",
    err
)]
pub(crate) async fn get_dimension() -> Result<u64> {
    let get_dimension_result: Result<u64, Error> =
        task::spawn_blocking(move || {
            // Setup sentence embeddings model
            let model = SentenceEmbeddingsBuilder::remote(
                SentenceEmbeddingsModelType::AllMiniLmL6V2,
            )
            .create_model()
            .map_err(|error| {
                tracing::error!("Failed to create model: {:?}", error);
                error
            })?;

            let dimension = model
                .get_embedding_dim()
                .map_err(|error| {
                    tracing::error!(
                        "Failed to get embedding dim: {:?}",
                        error
                    );
                    error
                })?;

            Ok(dimension as u64)
        })
        .await
        .map_err(|error| {
            tracing::error!(
                "Failed to spawn blocking task: {:?}",
                error
            );
            error
        })?;

    get_dimension_result
}
