use qdrant_client::prelude::{QdrantClient, QdrantClientConfig};
use anyhow::Result;
use url::Url;

pub struct QdrantCloudDB {
    pub client: QdrantClient,
    pub collection: String,
}

impl QdrantCloudDB {
    pub async fn new(collection: String, cluster_url: Url, api_key: String) -> Result<Self> {
        let mut config = QdrantClientConfig::from_url(cluster_url.as_str());
        config.set_api_key(&api_key);
        let client = QdrantClient::new(Some(config)).await?;
        Ok(Self {
            client,
            collection,
        })
    }
}