pub use tiktoken_rs::*;
use anyhow::Result;
use crate::utils::token::CountToken;

#[derive(Clone)]
#[readonly::make]
pub struct Tiktoken {
    #[readonly]
    pub model: String,
    #[readonly]
    pub bpe: CoreBPE,
}

impl Tiktoken {
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
