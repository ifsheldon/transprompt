use std::collections::HashMap;
use anyhow::Result;
use async_openai::types::ChatCompletionRequestMessage;
pub use tiktoken_rs::{get_bpe_from_model, CoreBPE};

use crate::utils::token::CountToken;
use lazy_static::lazy_static;

const TOKENS_PER_MESSAGE: usize = 3;
const TOKENS_PER_NAME: usize = 1;

lazy_static! {
    /// const map from model name to max tokens.
    /// TODO: when `LazyCell` is stabilized, use that instead
    pub static ref MODEL_TO_MAX_TOKENS: HashMap<&'static str, usize> = HashMap::from([
        ("gpt-4", 8192),
        ("gpt-4-0613", 8192),
        ("gpt-4-32k", 32768),
        ("gpt-4-32k-0613", 32768),
        ("gpt-3.5-turbo", 4096),
        ("gpt-3.5-turbo-16k", 16384),
        ("gpt-3.5-turbo-0613", 4096),
        ("gpt-3.5-turbo-16k-0613", 16384),
    ]);
}

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
        assert!(MODEL_TO_MAX_TOKENS.contains_key(model.as_str()), "model {} is not supported", model);
        let model = if model.starts_with("gpt-4-32k") {
            "gpt-4-32k"
        } else if model.starts_with("gpt-4") {
            "gpt-4"
        } else if model.starts_with("gpt-3.5") {
            "gpt-3.5-turbo"
        } else {
            unreachable!()
        };
        get_bpe_from_model(model).and_then(|bpe| Ok(Tiktoken {
            model: model.to_string(),
            bpe,
        }))
    }

    /// Count the number of tokens in a chat message. Following best practices from the OpenAI exmaple.
    ///
    /// Assuming the model is NOT the legacy `gpt-3.5-turbo-0301`
    ///
    /// TODO: use `tiktoken_rs::async_openai::get_chat_completion_max_tokens` when it adds newer model variants
    pub fn count_msg_token(&self, msg: &ChatCompletionRequestMessage) -> usize {
        let mut token_count = msg.content.as_ref()
            .and_then(|msg_string| Some(self.count_token(msg_string)))
            .unwrap_or(0);
        if msg.name.is_some() {
            token_count += TOKENS_PER_NAME;
        }
        token_count += TOKENS_PER_MESSAGE;
        return token_count;
    }

    #[inline]
    pub fn truncate_messages(&self,
                             messages: &Vec<ChatCompletionRequestMessage>,
                             system_message: Option<ChatCompletionRequestMessage>) -> Vec<ChatCompletionRequestMessage> {
        self.truncate_messages_with_max_tokens(messages, system_message, *MODEL_TO_MAX_TOKENS.get(self.model.as_str()).unwrap())
    }

    pub fn truncate_messages_with_max_tokens(&self,
                                             messages: &Vec<ChatCompletionRequestMessage>,
                                             system_message: Option<ChatCompletionRequestMessage>,
                                             max_tokens: usize) -> Vec<ChatCompletionRequestMessage> {
        assert_ne!(max_tokens, 0, "max_tokens cannot be 0");
        let mut trimmed_messages = Vec::with_capacity(messages.len());
        let mut token_count = 0;
        if let Some(system_message) = system_message {
            let system_message_token_count = self.count_msg_token(&system_message);
            assert!(system_message_token_count <= max_tokens, "system message token count {} is greater than max tokens {} of model {}", system_message_token_count, max_tokens, self.model);
            trimmed_messages.push(system_message);
            token_count += system_message_token_count;
        }
        // TODO: make this algorithm more smart as in Python `tokentrim`
        let mut start_i = messages.len();
        for i in (0..messages.len()).rev() {
            let message = &messages[i];
            let message_token_count = self.count_msg_token(message);
            if token_count + message_token_count > max_tokens {
                start_i = i + 1;
                break;
            }
            token_count += message_token_count;
            trimmed_messages.push(message.clone());
        }
        trimmed_messages.extend_from_slice(&messages[start_i..]);
        return trimmed_messages;
    }
}

impl CountToken for Tiktoken {
    fn count_token(&self, string: &str) -> usize {
        self.bpe.encode_with_special_tokens(string).len()
    }
}
