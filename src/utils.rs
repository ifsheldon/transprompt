pub mod vec_stores;
pub mod retrievers;
pub mod token;
pub mod llm;
pub mod postprocess;
pub(crate) mod prompt_processing;

use serde_json::{Map, Value};

pub type JsonMap = Map<String, Value>;