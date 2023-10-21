use anyhow::Result;
use async_openai::Client;
use async_openai::types::{CreateEmbeddingRequest, EmbeddingUsage};
use async_openai::types::EmbeddingInput;
use async_trait::async_trait;

/// Vector of floats representing an embedding.
pub type EmbedVec = Vec<f32>;

//TODO: when negative trait bound is implemented, add blanket AsyncSimplyEmbed impl for AsyncEmbed
//TODO: when async fn in trait is implemented, remove async_trait macro

/// Trait for getting the embedding dimension.
pub trait GetEmbedDim {
    fn embedding_dim(&self) -> Option<usize>;
}

/// Trait for embedding a string and outputting the embedding vector and extra info.
pub trait Embed: GetEmbedDim {
    type OutputExtra;
    fn embed(&self, string: impl Into<String>) -> Result<(EmbedVec, Self::OutputExtra)>;
}

/// Trait for embedding a string and outputting the embedding vector.
pub trait SimplyEmbed: GetEmbedDim {
    fn embed(&self, string: impl Into<String>) -> Result<EmbedVec>;
}

/// Blanket impl of SimplyEmbed for Embed trait.
impl<T> SimplyEmbed for T where T: Embed {
    fn embed(&self, string: impl Into<String>) -> Result<EmbedVec> {
        Embed::embed(self, string).map(|e| e.0)
    }
}

/// Async version of Embed trait.
#[async_trait]
pub trait AsyncEmbed: GetEmbedDim {
    type OutputExtra;
    async fn embed(&self, string: impl Into<String> + Send) -> Result<(EmbedVec, Self::OutputExtra)>;
}

/// Async version of SimplyEmbed trait.
#[async_trait]
pub trait AsyncSimplyEmbed: GetEmbedDim {
    async fn embed(&self, string: impl Into<String> + Send) -> Result<EmbedVec>;
}

/// Blanket impl of AsyncSimplyEmbed for AsyncEmbed trait.
#[async_trait]
impl<T: SimplyEmbed + Sync> AsyncSimplyEmbed for T {
    async fn embed(&self, string: impl Into<String> + Send) -> Result<EmbedVec> {
        SimplyEmbed::embed(self, string)
    }
}

/// Blanket impl of AsyncEmbed for Embed trait.
#[async_trait]
impl<T: Embed + Sync> AsyncEmbed for T {
    type OutputExtra = T::OutputExtra;
    async fn embed(&self, string: impl Into<String> + Send) -> Result<(EmbedVec, Self::OutputExtra)> {
        Embed::embed(self, string)
    }
}


/// Embedding model from OpenAI API.
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
    /// send a request to the OpenAI API to embed a string. Returns the embedding vector and embedding usage, or an error.
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
