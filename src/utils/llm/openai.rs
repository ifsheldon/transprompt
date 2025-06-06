use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use crate::utils::helper_traits::{ThenDo, ThenDoMut};
use crate::utils::token::tiktoken::{Tiktoken, MODEL_TO_MAX_TOKENS};
use crate::utils::JsonMap;
use async_openai_wasm::error::OpenAIError;
use async_openai_wasm::types::{
    ChatCompletionFunctionCall, ChatCompletionFunctions, ChatCompletionMessageToolCall,
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionResponseStream, ChatCompletionStreamResponseDelta, ChatCompletionToolType,
    CreateChatCompletionRequest, CreateChatCompletionResponse, FunctionCall, Stop,
};
use async_openai_wasm::Client;
use async_openai_wasm::config::Config;
use serde::{Deserialize, Serialize};

/// Configuration for OpenAI LLM in a conversation setting. Partially copied from [async_openai::types::CreateChatCompletionRequest].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationConfig {
    /// ID of the model to use.
    /// See the [model endpoint compatibility](https://platform.openai.com/docs/models/model-endpoint-compatibility) table for details on which models work with the Chat API.
    pub model: String,

    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    ///
    /// We generally recommend altering this or `top_p` but not both.
    pub temperature: Option<f32>, // min: 0, max: 2, default: 1,

    /// An alternative to sampling with temperature, called nucleus sampling,
    /// where the model considers the results of the tokens with top_p probability mass.
    /// So 0.1 means only the tokens comprising the top 10% probability mass are considered.
    ///
    ///  We generally recommend altering this or `temperature` but not both.
    pub top_p: Option<f32>, // min: 0, max: 1, default: 1

    /// How many chat completion choices to generate for each input message.
    pub n: Option<u8>, // min:1, max: 128, default: 1

    /// Up to 4 sequences where the API will stop generating further tokens.
    pub stop: Option<Stop>,

    /// The maximum number of [tokens](https://platform.openai.com/tokenizer) to generate in the chat completion.
    ///
    /// The total length of input tokens and generated tokens is limited by the model's context length. [Example Python code](https://github.com/openai/openai-cookbook/blob/main/examples/How_to_count_tokens_with_tiktoken.ipynb) for counting tokens.
    pub max_tokens: Option<u16>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    ///
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/api-reference/parameter-details)
    pub presence_penalty: Option<f32>, // min: -2.0, max: 2.0, default 0

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
    ///
    /// [See more information about frequency and presence penalties.](https://platform.openai.com/docs/api-reference/parameter-details)
    pub frequency_penalty: Option<f32>, // min: -2.0, max: 2.0, default: 0

    /// Modify the likelihood of specified tokens appearing in the completion.
    ///
    /// Accepts a json object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100.
    /// Mathematically, the bias is added to the logits generated by the model prior to sampling.
    /// The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection;
    /// values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
    pub logit_bias: Option<HashMap<String, serde_json::Value>>, // default: null

    /// A unique identifier representing your end-user, which can help OpenAI to monitor and detect abuse. [Learn more](https://platform.openai.com/docs/guides/safety-best-practices/end-user-ids).
    pub user: Option<String>,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
            max_tokens: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
        }
    }
}

/// A message in a conversation with optional metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMsg {
    pub msg: ChatCompletionRequestMessage,
    pub metadata: Option<JsonMap>,
}

impl Display for ChatMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.msg).unwrap())
    }
}

impl ChatMsg {
    pub fn merge_delta(&mut self, delta: &ChatCompletionStreamResponseDelta) -> (bool, bool) {
        match self.msg {
            ChatCompletionRequestMessage::Assistant(ref mut msg) => {
                // if we have a function call delta, we need to update the function call
                let mut function_call_updated = false;
                delta.function_call.ok_then_do(|fn_call_delta| {
                    msg.function_call.ok_then_do_otherwise_mut(
                        |func_call| {
                            // if the container message already has a function call, we need to update the function call
                            fn_call_delta
                                .name
                                .ok_then_do(|fn_name| func_call.name = fn_name.clone());
                            fn_call_delta
                                .arguments
                                .ok_then_do(|fn_args| func_call.arguments.push_str(fn_args));
                        },
                        |func_call_option| {
                            // if the container message does not have a function call, we need to create one
                            *func_call_option = Some(FunctionCall {
                                name: fn_call_delta
                                    .name
                                    .as_ref()
                                    .map_or_else(String::new, Clone::clone),
                                arguments: fn_call_delta
                                    .arguments
                                    .as_ref()
                                    .map_or_else(String::new, Clone::clone),
                            });
                        },
                    );
                    function_call_updated = true;
                });

                let mut tool_call_updated = false;
                delta.tool_calls.ok_then_do(|tool_call_deltas| {
                    // TODO: test this
                    msg.tool_calls.ok_then_do_otherwise_mut(
                        |tool_calls| {
                            // if the container message already has a tool call, we need to update the tool call
                            tool_call_deltas.iter().for_each(|tool_call_delta| {
                                let index = tool_call_delta.index as usize;
                                if let Some(tool_call) = tool_calls.get_mut(index) {
                                    tool_call_delta.id.ok_then_do(|id| {
                                        assert_eq!(
                                            id.as_str(),
                                            tool_call.id.as_str(),
                                            "Tool call id mismatch"
                                        )
                                    });
                                    tool_call_delta.function.ok_then_do(|function| {
                                        function.name.ok_then_do(|name| {
                                            assert_eq!(
                                                name.as_str(),
                                                tool_call.function.name.as_str(),
                                                "Tool call function name mismatch"
                                            )
                                        });
                                        function.arguments.ok_then_do(|args| {
                                            tool_call.function.arguments.push_str(args)
                                        });
                                    });
                                } else {
                                    log::error!(
                                        "Impossible Tool call index out of bound: index={}",
                                        index
                                    );
                                }
                            });
                        },
                        |tool_calls_option| {
                            // if the container message does not have a tool call, we need to create one
                            let max_index = tool_call_deltas
                                .iter()
                                .map(|tool_call_delta| tool_call_delta.index)
                                .max()
                                .unwrap();
                            let tool_calls_num = max_index + 1;
                            let mut tool_calls = Vec::with_capacity(tool_calls_num as usize);
                            const PLACE_HOLDER: String = String::new();
                            for _ in 0..tool_calls_num {
                                tool_calls.push(ChatCompletionMessageToolCall {
                                    id: PLACE_HOLDER.clone(),
                                    r#type: ChatCompletionToolType::default(),
                                    function: FunctionCall {
                                        name: PLACE_HOLDER.clone(),
                                        arguments: PLACE_HOLDER.clone(),
                                    },
                                });
                            }
                            tool_call_deltas.iter().for_each(|tool_call_delta| {
                                let index = tool_call_delta.index as usize;
                                if let Some(tool_call) = tool_calls.get_mut(index) {
                                    tool_call_delta
                                        .id
                                        .ok_then_do(|id| tool_call.id = id.clone());
                                    tool_call_delta.function.ok_then_do(|function| {
                                        function.name.ok_then_do(|name| {
                                            tool_call.function.name = name.clone()
                                        });
                                        function.arguments.ok_then_do(|arguments| {
                                            tool_call.function.arguments = arguments.clone()
                                        });
                                    });
                                } else {
                                    log::error!(
                                        "Impossible Tool call index out of bound: index={}",
                                        index
                                    );
                                }
                            });
                            *tool_calls_option = Some(tool_calls);
                        },
                    );
                    tool_call_updated = true;
                });

                // if we have a content delta, we need to update the content
                let mut content_updated = false;
                delta.content.ok_then_do(|content_delta| {
                    msg.content.ok_then_do_otherwise_mut(
                        |content| {
                            // if the container message already has a content, we need to update the content
                            match content {
                                ChatCompletionRequestAssistantMessageContent::Text(t) => {
                                    t.push_str(content_delta.as_str())
                                }
                                ChatCompletionRequestAssistantMessageContent::Array(parts) => parts
                                    .push(ChatCompletionRequestAssistantMessageContentPart::Text(
                                        ChatCompletionRequestMessageContentPartText {
                                            text: content_delta.clone(),
                                        },
                                    )),
                            }
                        },
                        |content_option| {
                            // if the container message does not have a content, we need to create one
                            *content_option = Some(
                                ChatCompletionRequestAssistantMessageContent::Text(content_delta.clone())
                            )
                        },
                    );
                    content_updated = true;
                });

                return (function_call_updated, content_updated);
            }
            _ => unreachable!("only assistant message will be streamed"),
        }
    }
}

/// A conversation with OpenAI LLM.
#[derive(Clone)]
pub struct Conversation {
    pub client: Client<Arc<dyn Config>>,
    pub configs: ConversationConfig,
    pub history: Vec<ChatMsg>,
    pub auto_truncate_history: bool,
    pub tiktoken: Tiktoken,
}

impl Display for Conversation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string_pretty(&self.history).unwrap()
        )
    }
}

impl Debug for Conversation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"
history: {}
auto_truncate_history: {}
"#,
            serde_json::to_string_pretty(&self.history).unwrap(),
            self.auto_truncate_history
        )
    }
}

impl Conversation {
    /// Create a new conversation with OpenAI LLM.
    pub fn new(client: Client<Arc<dyn Config>>, configs: ConversationConfig, auto_truncate_history: bool) -> Self {
        let tiktoken = Tiktoken::new(configs.model.clone()).unwrap();
        Self {
            client,
            configs,
            history: Vec::new(),
            auto_truncate_history,
            tiktoken,
        }
    }

    /// Count the number of tokens in the conversation history.
    pub fn count_tokens_history(&self) -> usize {
        self.history
            .iter()
            .map(|msg| self.tiktoken.count_msg_token(&msg.msg))
            .sum()
    }

    /// Insert a message into the conversation history.
    pub fn insert_history(
        &mut self,
        message: ChatCompletionRequestMessage,
        metadata: Option<JsonMap>,
    ) {
        self.history.push(ChatMsg {
            msg: message,
            metadata,
        });
        if self.auto_truncate_history {
            self.truncate_history();
        }
    }

    #[inline]
    fn create_chat_request(
        &self,
        functions: Option<Vec<ChatCompletionFunctions>>,
        function_call: Option<ChatCompletionFunctionCall>,
        stream: bool,
    ) -> CreateChatCompletionRequest {
        let config = self.configs.clone();
        CreateChatCompletionRequest {
            model: config.model,
            store: None,
            reasoning_effort: None,
            messages: self.history.iter().map(|msg| msg.msg.clone()).collect(),
            functions,
            function_call,
            temperature: config.temperature,
            top_p: config.top_p,
            tools: None,
            n: config.n,
            modalities: None,
            prediction: None,
            stream: if stream { Some(true) } else { None },
            stop: config.stop,
            max_tokens: config.max_tokens.map(|m| m as u32),
            presence_penalty: config.presence_penalty,
            response_format: None,
            frequency_penalty: config.frequency_penalty,
            logit_bias: config.logit_bias,
            logprobs: None,
            user: config.user,
            seed: None,
            tool_choice: None,
            top_logprobs: None,
            metadata: None,
            max_completion_tokens: None,
            audio: None,
            service_tier: None,
            stream_options: None,
            parallel_tool_calls: None,
            web_search_options: None,
            extra_params: None,
        }
    }

    /// Commit a chat request to OpenAI LLM with the current conversation history.
    pub async fn query_with_history(
        &self,
        functions: Option<Vec<ChatCompletionFunctions>>,
        function_call: Option<ChatCompletionFunctionCall>,
    ) -> Result<CreateChatCompletionResponse, OpenAIError> {
        let chat_request = self.create_chat_request(functions, function_call, false);
        self.client.chat().create(chat_request).await
    }

    /// Commit a chat request to OpenAI LLM with the current conversation history.
    /// Returns a stream
    pub async fn query_and_stream_with_history(
        &self,
        functions: Option<Vec<ChatCompletionFunctions>>,
        function_call: Option<ChatCompletionFunctionCall>,
    ) -> Result<ChatCompletionResponseStream, OpenAIError> {
        let chat_request = self.create_chat_request(functions, function_call, true);
        self.client.chat().create_stream(chat_request).await
    }

    pub fn truncate_history(&mut self) {
        let mut max_tokens = *MODEL_TO_MAX_TOKENS
            .get(self.configs.model.as_str())
            .unwrap();
        let sys_prompt = self
            .history
            .first()
            .and_then(|chat_msg| match &chat_msg.msg {
                ChatCompletionRequestMessage::System(_prompt) => {
                    max_tokens -= self.tiktoken.count_msg_token(&chat_msg.msg);
                    Some(chat_msg)
                }
                _ => None,
            });
        let truncate_start_idx = self.tiktoken.get_truncate_start_idx(
            &self
                .history
                .iter()
                .map(|chat_msg| chat_msg.msg.clone())
                .collect(),
            max_tokens,
        );
        if truncate_start_idx > 0 {
            if let Some(sys_prompt) = sys_prompt {
                let mut new_history =
                    Vec::with_capacity(self.history.len() - truncate_start_idx + 1);
                new_history.push(sys_prompt.clone());
                new_history.extend_from_slice(&self.history[truncate_start_idx..]);
                self.history = new_history;
            } else {
                self.history = self.history[truncate_start_idx..].to_vec();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::{stdout, Write};

    use anyhow::Result;
    use async_openai_wasm::config::AzureConfig;
    use async_openai_wasm::types::{
        ChatCompletionFunctionsArgs, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, FinishReason,
    };
    use async_openai_wasm::Client;
    use futures::StreamExt;
    use serde_json::json;

    use crate::utils::helper_traits::ThenDo;
    use crate::utils::llm::openai::ChatMsg;

    #[derive(Debug, Clone, serde::Deserialize)]
    struct WeatherFunctionArguments {
        location: String,
        unit: Option<String>,
    }

    fn print_immediately(msg: impl Into<String>) {
        print!("{}", msg.into());
        stdout().flush().unwrap();
    }

    #[tokio::test]
    async fn test_merge_delta() -> Result<()> {
        // read configs from file
        let azure_test_configs: AzureConfig =
            serde_json::from_str(std::fs::read_to_string(".azure_configs.json")?.as_str())?;
        // adapted the code from https://github.com/64bit/async-openai/blob/main/examples/function-call-stream/src/main.rs
        let client = Client::with_config(azure_test_configs);

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo-0613")
            .messages([ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content("What's the weather like in Boston?")
                    .build()?,
            )])
            .functions([ChatCompletionFunctionsArgs::default()
                .name("get_current_weather")
                .description("Get the current weather in a given location")
                .parameters(json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA",
                        },
                        "unit": { "type": "string", "enum": ["celsius", "fahrenheit"] },
                    },
                    "required": ["location"],
                }))
                .build()?])
            .function_call("auto")
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;
        let mut fn_name = String::new();
        let mut fn_args = String::new();
        let mut function_called = false;
        let mut assistant_message = ChatMsg {
            msg: ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessageArgs::default().build()?,
            ),
            metadata: None,
        };
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(delta) => {
                    if delta.choices.is_empty() {
                        // the first chunk is somehow empty
                        print_immediately(format!("No choices in chunk\nChunk: {:?}\n", delta));
                        continue;
                    }
                    print_immediately(format!("Chunk: {:?}\n", delta));
                    let chat_choice = &delta.choices[0];
                    assistant_message.merge_delta(&chat_choice.delta);
                    chat_choice.delta.function_call.ok_then_do(|fn_call| {
                        print_immediately(format!("function_call: {:?}\n", fn_call));
                        fn_call.name.ok_then_do(|name| fn_name = name.clone());
                        fn_call.arguments.ok_then_do(|args| fn_args.push_str(args));
                    });
                    if let Some(finish_reason) = &chat_choice.finish_reason {
                        if *finish_reason == FinishReason::FunctionCall {
                            print_immediately("\nfunction called\n");
                            function_called = true;
                        }
                    } else if let Some(content) = &chat_choice.delta.content {
                        print_immediately(format!("content: {}", content));
                    }
                }
                Err(err) => {
                    print_immediately(format!("Error: {}", err));
                }
            }
            stdout().flush()?;
        }
        print_immediately(format!("fn_name: {}\nfn_args: {}\n", fn_name, fn_args));

        match assistant_message.msg {
            ChatCompletionRequestMessage::Assistant(msg) => {
                assert_eq!(fn_name, msg.function_call.as_ref().unwrap().name.as_str());
                assert_eq!(
                    fn_args,
                    msg.function_call.as_ref().unwrap().arguments.as_str()
                );
            }
            _ => unreachable!(),
        }

        let fn_args: WeatherFunctionArguments = serde_json::from_str(fn_args.as_str())?;
        assert_eq!(fn_name, "get_current_weather");
        assert!(function_called);
        assert!(fn_args.location.to_lowercase().contains("boston"));

        Ok(())
    }
}
