use std::collections::HashMap;

use anyhow::{Ok, Result};
use chrono::{DateTime, Utc};
use qdrant_client::{
    prelude::{Payload, QdrantClient},
    qdrant::{
        vectors_config::Config, CreateCollection, Distance, Filter,
        PointStruct, ScoredPoint, SearchPoints, Value, VectorParams,
        VectorsConfig,
    },
};

use crate::vector_db::embeddings;

pub(crate) struct DataBase {
    pub(crate) client: QdrantClient,
    pub(crate) name: String,
    pub(crate) index: u64,
}

pub(crate) struct MetaData {
    pub(crate) datetime: DateTime<Utc>,
    pub(crate) author: String,
}

impl MetaData {
    fn to_payload(&self) -> Payload {
        let mut map = HashMap::new();
        map.insert(
            "datetime".to_string(),
            Value::from(
                self.datetime
                    .format("%Y-%m-%dT%H:%M:%S%.3f")
                    .to_string(),
            ),
        );
        map.insert(
            "author".to_string(),
            Value::from(self.author.clone()),
        );

        Payload::new_from_hashmap(map)
    }
}

impl DataBase {
    #[tracing::instrument(
        name = "vector_db.database.reset",
        err,
        skip(self, request)
    )]
    pub(crate) async fn reset(&self) -> Result<()> {
        self.client
            .delete_collection(self.name)
            .await?;

        self.client
            .create_collection(&CreateCollection {
                collection_name: self.name.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 10,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "vector_db.database.upsert",
        err,
        skip(self, request)
    )]
    pub(crate) async fn upsert(
        &mut self,
        text: &str,
        meta_data: MetaData,
    ) -> Result<()> {
        let embedding = embeddings::embed(text)?;
        let payload = meta_data.to_payload();
        let point = PointStruct::new(self.index, embedding, payload);
        self.index += 1;

        self.client
            .upsert_points(self.name, vec![point], None)
            .await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "vector_db.database.search",
        err,
        skip(self, request)
    )]
    pub(crate) async fn search(
        &self,
        query: &str,
        count_limit: u64,
        filter: Option<Filter>,
    ) -> Result<Vec<ScoredPoint>> {
        let embedding = embeddings::embed(query)?;
        let result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.name.to_string(),
                vector: embedding,
                limit: count_limit,
                filter,
                with_payload: Some(true.into()),
                with_vectors: Some(true.into()),
                ..Default::default()
            })
            .await?;

        Ok(result.result)
    }
}
