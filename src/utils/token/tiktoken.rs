use anyhow::Result;
pub use tiktoken_rs::*;

use crate::utils::token::CountToken;

/// Counter using the Tiktoken tokenizer.
#[derive(Clone)]
#[readonly::make]
pub struct Tiktoken {
    /// The model name of the tokenizer. read-only.
    #[readonly]
    pub model: String,
    /// The tokenizer. read-only.
    #[readonly]
    pub bpe: CoreBPE,
}

impl Tiktoken {
    /// Create a new Tiktoken counter.
    pub fn new(model: impl Into<String>) -> Result<Self> {
        let model = model.into();
        get_bpe_from_model(&model).and_then(|bpe| Ok(Tiktoken {
            model,
            bpe,
        }))
    }
}

impl CountToken for Tiktoken {
    fn count_token(&self, string: &str) -> usize {
        self.bpe.encode_with_special_tokens(string).len()
    }
}
