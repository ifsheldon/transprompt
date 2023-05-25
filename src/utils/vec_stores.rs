use qdrant_client::prelude::{CreateCollection, Distance, QdrantClient, QdrantClientConfig, SearchPoints};
use anyhow::Result;
use qdrant_client::qdrant::{CollectionOperationResponse, PointStruct, ScoredPoint, VectorParams, VectorsConfig, WithPayloadSelector};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable;
use url::Url;
use crate::utils::embedding::EmbedVec;
use crate::utils::JsonMap;

pub type Vector = EmbedVec;

pub struct QdrantCloudDB {
    pub client: QdrantClient,
    pub collection: String,
}

impl QdrantCloudDB {
    pub fn create_point(vec: Vector, metadata: JsonMap) -> PointStruct {
        let metadata = metadata.into_iter()
            .map(|(string, val)| (string, val.into()))
            .collect();
        PointStruct {
            id: None,
            payload: metadata,
            vectors: Some(vec.into()),
        }
    }

    pub async fn new(collection: String, cluster_url: Url, api_key: String) -> Result<Self> {
        let mut config = QdrantClientConfig::from_url(cluster_url.as_str());
        config.set_api_key(&api_key);
        let client = QdrantClient::new(Some(config)).await?;
        Ok(Self {
            client,
            collection,
        })
    }

    pub async fn create_simple_vector_collection(&self,
                                                 collection_name: impl Into<String>,
                                                 distance: Distance,
                                                 vector_size: u64) -> Result<CollectionOperationResponse> {
        let create = CreateCollection {
            collection_name: collection_name.into(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: vector_size,
                    distance: distance.into(),
                    hnsw_config: None,
                    quantization_config: None,
                }))
            }),
            ..Default::default()
        };
        self.client.create_collection(&create).await
    }

    pub async fn upsert_point(&self, vec: Vector, metadata: JsonMap) -> Result<()> {
        self.upsert_points(vec![(vec, metadata)]).await
    }

    pub async fn upsert_points(&self, points: Vec<(Vector, JsonMap)>) -> Result<()> {
        let points = points.into_iter()
            .map(|(v, m)| Self::create_point(v, m))
            .collect();
        self.client.upsert_points(&self.collection, points, None).await.map(|_| ())
    }

    pub async fn search_nearest_with_metadata(&self, vec: Vector, top_k: u64) -> Result<Vec<ScoredPoint>> {
        self.client.search_points(&SearchPoints {
            collection_name: self.collection.clone(),
            vector: vec,
            filter: None,
            limit: top_k,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(Enable(true)),
            }),
            params: None,
            score_threshold: None,
            offset: None,
            vector_name: None,
            with_vectors: None,
            read_consistency: None,
        }).await.map(|response| response.result)
    }
}