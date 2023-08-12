use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

#[tracing::instrument(name = "vector_db.embed", err)]
pub(crate) async fn embed(sentence: String) -> Result<Vec<Vec<f32>>> {
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
            tracing::error!(
                "Failed to generate embeddings: {:?}",
                error
            );
            error
        })?;

    Ok(embeddings)
}
