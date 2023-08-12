use std::{collections::HashMap, fmt::Formatter};

use anyhow::Result;
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

#[derive(Debug)]
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

pub(crate) struct DataBase {
    pub(crate) client: QdrantClient,
    pub(crate) name: String,
    pub(crate) index: u64,
}

impl std::fmt::Debug for DataBase {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("DataBase")
            .field("client", &self.client.cfg.uri)
            .field("name", &self.name)
            .field("index", &self.index)
            .finish()
    }
}

impl DataBase {
    #[tracing::instrument(
        name = "vector_db.database.reset",
        err,
        skip(self)
    )]
    pub(crate) async fn reset(&self) -> Result<()> {
        self.client
            .delete_collection(self.name.to_string())
            .await
            .map_err(|error| {
                tracing::error!(
                    "Failed to delete collection: {:?}",
                    error
                );
                error
            })?;

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
            .await
            .map_err(|error| {
                tracing::error!(
                    "Failed to create collection: {:?}",
                    error
                );
                error
            })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "vector_db.database.upsert",
        err,
        skip(self, text, meta_data)
    )]
    pub(crate) async fn upsert(
        &mut self,
        text: String,
        meta_data: MetaData,
    ) -> Result<()> {
        let embedding = embeddings::embed(text)
            .await
            .map_err(|error| {
                tracing::error!("Failed to embed text: {:?}", error);
                error
            })?;
        let vector = embedding[0].clone();
        let payload = meta_data.to_payload();
        let point = PointStruct::new(self.index, vector, payload);
        self.index += 1;

        self.client
            .upsert_points(self.name.clone(), vec![point], None)
            .await
            .map_err(|error| {
                tracing::error!("Failed to upsert points: {:?}", error);
                error
            })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "vector_db.database.search",
        err,
        skip(self, query, count_limit, filter)
    )]
    pub(crate) async fn search(
        &self,
        query: String,
        count_limit: u64,
        filter: Option<Filter>,
    ) -> Result<Vec<ScoredPoint>> {
        let embedding = embeddings::embed(query)
            .await
            .map_err(|error| {
                tracing::error!("Failed to embed query: {:?}", error);
                error
            })?;
        let vector = embedding[0].clone();
        let result = self
            .client
            .search_points(&SearchPoints {
                collection_name: self.name.clone(),
                vector,
                limit: count_limit,
                filter,
                with_payload: Some(true.into()),
                with_vectors: Some(true.into()),
                ..Default::default()
            })
            .await
            .map_err(|error| {
                tracing::error!("Failed to search points: {:?}", error);
                error
            })?;

        Ok(result.result)
    }
}
