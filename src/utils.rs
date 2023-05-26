//! # Utilities
//!
//! Including:
//! * Vector storage
//! * Token counters and tokenizers
//! * LLM
//! * Postprocess for strings
//! * Timing utilities for virtual time

pub mod vec_stores;
pub mod token;
pub mod llm;
pub mod postprocess;
pub mod embedding;
pub mod timing;
pub(crate) mod prompt_processing;

use serde_json::{Map, Value};

pub type JsonMap = Map<String, Value>;