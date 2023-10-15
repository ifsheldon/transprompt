//! # Utilities
//!
//! Including:
//! * Vector storage
//! * Token counters and tokenizers
//! * LLM
//! * Postprocess for strings
//! * Timing utilities for virtual time

use serde_json::{Map, Value};

pub mod vec_stores;
pub mod token;
pub mod llm;
pub mod postprocess;
pub mod embedding;
#[cfg(feature = "terminal_printing")]
pub mod printing;
pub(crate) mod prompt_processing;
pub(crate) mod helper_traits;

pub type JsonMap = Map<String, Value>;