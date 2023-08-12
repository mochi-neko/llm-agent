use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};

#[tracing::instrument(
    name = "vector_db.embed",
    err,
    skip(self, request)
)]
pub(crate) async fn embed(sentence: String) -> Result<Vec<Vec<String>>> {
    // Set-up sentence embeddings model
    let model = SentenceEmbeddingsBuilder::remote(
        SentenceEmbeddingsModelType::AllMiniLmL12V2,
    )
    .create_model()
    .map_err(|error| {
        tracing::error!("Failed to create model: {:?}", error);
        error
    })?;

    // Generate Embeddings
    let embeddings = model
        .encode(&sentence)
        .map_err(|error| {
            tracing::error!(
                "Failed to generate embeddings: {:?}",
                error
            );
            error
        })?;

    Ok(embeddings)
}
