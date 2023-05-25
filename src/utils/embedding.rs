use anyhow::Result;
use async_openai::Client;
use async_openai::types::{CreateEmbeddingRequest, EmbeddingUsage};
use async_openai::types::EmbeddingInput;
use async_trait::async_trait;

pub type EmbedVec = Vec<f32>;

//TODO: when negative trait bound is implemented, add blanket AsyncSimplyEmbed impl for AsyncEmbed
//TODO: when async fn in trait is implemented, remove async_trait macro

pub trait GetEmbedDim {
    fn embedding_dim(&self) -> Option<usize>;
}

pub trait Embed: GetEmbedDim {
    type OutputExtra;
    fn embed(&self, string: impl Into<String>) -> Result<(EmbedVec, Self::OutputExtra)>;
}


pub trait SimplyEmbed: GetEmbedDim {
    fn embed(&self, string: impl Into<String>) -> Result<EmbedVec>;
}


impl<T> SimplyEmbed for T where T: Embed {
    fn embed(&self, string: impl Into<String>) -> Result<EmbedVec> {
        Embed::embed(self, string).map(|e| e.0)
    }
}

#[async_trait]
pub trait AsyncEmbed: GetEmbedDim {
    type OutputExtra;
    async fn embed(&self, string: impl Into<String> + Send) -> Result<(EmbedVec, Self::OutputExtra)>;
}

#[async_trait]
pub trait AsyncSimplyEmbed: GetEmbedDim {
    async fn embed(&self, string: impl Into<String> + Send) -> Result<EmbedVec>;
}


#[async_trait]
impl<T: SimplyEmbed + Sync> AsyncSimplyEmbed for T {
    async fn embed(&self, string: impl Into<String> + Send) -> Result<EmbedVec> {
        SimplyEmbed::embed(self, string)
    }
}

#[async_trait]
impl<T: Embed + Sync> AsyncEmbed for T {
    type OutputExtra = T::OutputExtra;
    async fn embed(&self, string: impl Into<String> + Send) -> Result<(EmbedVec, Self::OutputExtra)> {
        Embed::embed(self, string)
    }
}


#[derive(Clone, Debug)]
pub struct OpenAIEmbedding {
    pub client: Client,
    pub embedding_model: String,
}

impl GetEmbedDim for OpenAIEmbedding {
    fn embedding_dim(&self) -> Option<usize> {
        let dim = match self.embedding_model.as_str() {
            "text-embedding-ada-002" => 1536,
            e if e.contains("ada") => 1024,
            e if e.contains("babbage") => 2048,
            e if e.contains("curie") => 4096,
            e if e.contains("davinci") => 12288,
            _ => panic!("Embedding model {} is not in the list", self.embedding_model)
        };
        Some(dim)
    }
}

impl OpenAIEmbedding {
    async fn request_embed(&self, string: impl Into<String>) -> Result<(Vec<f32>, EmbeddingUsage)> {
        let request = CreateEmbeddingRequest {
            model: self.embedding_model.clone(),
            input: EmbeddingInput::String(string.into()),
            user: None,
        };
        let mut response = self.client.embeddings().create(request).await?;
        let emb = response.data.pop().unwrap().embedding;
        let usage = response.usage;
        Ok((emb, usage))
    }
}

#[async_trait]
impl AsyncEmbed for OpenAIEmbedding {
    type OutputExtra = EmbeddingUsage;

    async fn embed(&self, string: impl Into<String> + Send) -> Result<(EmbedVec, Self::OutputExtra)> {
        self.request_embed(string).await
    }
}

#[async_trait]
impl AsyncSimplyEmbed for OpenAIEmbedding {
    async fn embed(&self, string: impl Into<String> + Send) -> Result<EmbedVec> {
        self.request_embed(string).await.map(|(emb, _)| emb)
    }
}
