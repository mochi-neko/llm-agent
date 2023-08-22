use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    // Build protobuf
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_server(true)
        .file_descriptor_set_path(
            out_dir
                .clone()
                .join("creature_descriptor.bin"),
        )
        .out_dir(out_dir)
        .compile(&["proto/creature.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("protobuf compile error: {:?}", e));

    // Setup sentence embeddings model
    let _model = SentenceEmbeddingsBuilder::remote(
        SentenceEmbeddingsModelType::AllMiniLmL6V2,
    )
    .create_model()?;

    Ok(())
}
