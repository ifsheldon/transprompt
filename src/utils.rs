pub mod vec_stores;
pub mod retrievers;
pub mod string;
pub mod token;
pub mod llm;
pub mod postprocess;

use serde_json::{Map, Value};

pub type JsonMap = Map<String, Value>;