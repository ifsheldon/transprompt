use std::collections::HashMap;
use anyhow::Result;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContent};
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
        let mut token_count = match msg {
            ChatCompletionRequestMessage::System(msg) => msg.content.as_ref().map_or(0, |msg| self.count_token(msg)),
            ChatCompletionRequestMessage::User(msg) => msg.content.as_ref().map_or(0, |msg| {
                match msg {
                    ChatCompletionRequestUserMessageContent::Text(s) => self.count_token(s),
                    ChatCompletionRequestUserMessageContent::Array(_) => todo!()
                }
            }),
            ChatCompletionRequestMessage::Assistant(msg) => msg.content.as_ref().map_or(0, |msg| self.count_token(msg)),
            ChatCompletionRequestMessage::Tool(_) => unimplemented!("tool message is not supported due to lack of details from OpenAI"),
            ChatCompletionRequestMessage::Function(_) => unimplemented!("function message is not supported due to lack of details from OpenAI")
        };
        // TODO: count name tokens when it is supported again
        // if msg.name.is_some() {
        //     token_count += TOKENS_PER_NAME;
        // }
        token_count += TOKENS_PER_MESSAGE;
        return token_count;
    }

    #[inline]
    pub fn truncate_messages(&self,
                             messages: &Vec<ChatCompletionRequestMessage>,
                             system_message: Option<ChatCompletionRequestMessage>) -> Vec<ChatCompletionRequestMessage> {
        if messages.is_empty() {
            return messages.clone();
        }
        let max_tokens = *MODEL_TO_MAX_TOKENS.get(self.model.as_str()).unwrap();
        return if let Some(sys_prompt) = system_message {
            let sys_prompt_token_count = self.count_msg_token(&sys_prompt);
            assert!(sys_prompt_token_count <= max_tokens, "system message token count {} is greater than max tokens {}", sys_prompt_token_count, max_tokens);
            let truncate_start_idx = self.get_truncate_start_idx(messages, max_tokens - sys_prompt_token_count);
            if truncate_start_idx == 0 {
                let mut new_messages = messages.clone();
                if !messages.first().unwrap().eq(&sys_prompt) {
                    new_messages[0] = sys_prompt;
                }
                new_messages
            } else {
                let mut new_messages = Vec::with_capacity(messages.len() - truncate_start_idx + 1);
                new_messages.push(sys_prompt);
                new_messages.extend_from_slice(&messages[truncate_start_idx..]);
                new_messages
            }
        } else {
            let truncate_start_idx = self.get_truncate_start_idx(messages, max_tokens);
            if truncate_start_idx == 0 {
                messages.clone()
            } else {
                messages[truncate_start_idx..].to_vec()
            }
        };
    }

    pub(crate) fn get_truncate_start_idx(&self,
                                         messages: &Vec<ChatCompletionRequestMessage>,
                                         max_tokens: usize) -> usize {
        if messages.is_empty() {
            return 0;
        }
        let num_messages = messages.len();
        if max_tokens == 0 {
            return num_messages;
        }
        let mut token_count = 0;
        // TODO: make this algorithm more smart as in Python `tokentrim`
        let mut truncate_start_idx = 0;
        for (idx, msg) in messages.iter().enumerate().rev() {
            let message_token_count = self.count_msg_token(msg);
            if token_count + message_token_count > max_tokens {
                truncate_start_idx = idx + 1;
                break;
            }
            token_count += message_token_count;
        }
        return truncate_start_idx;
    }
}

impl CountToken for Tiktoken {
    fn count_token(&self, string: &str) -> usize {
        self.bpe.encode_with_special_tokens(string).len()
    }
}
