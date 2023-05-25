//! # Prompt
//! A prompt is simply a string
//! ## PromptTemplate
//! A prompt template is a string with placeholders. It can also have metadata in JSON format.
//!
//! ## Placeholder
//! A placeholder is a string that is in the format of `{[name]}`. It can be filled with a value.
//! It has a name, which is the string inside the square brackets.
//!
//! ## PartialPrompt
//! A partial prompt is a prompt template with some placeholders filled. A partial prompt can be only constructed from a prompt template via [PromptTemplate::construct_prompt].
//!
//! The placeholders in a partial prompt can be filled with values via [PartialPrompt::fill] or [PartialPrompt::try_fill]. You can also use these two methods to update the filling values of the placeholders.
//! When all placeholders are filled, the partial prompt can be completed via [PartialPrompt::complete], in which the placeholders in a template are **actually** replaced with the filling values.
//!
//! ### Counting tokens
//! A partial prompt can be used to count the number of tokens in the prompt. This is useful when you want to limit the number of tokens in the prompt. For simple counting of tokens, you can use [PartialPrompt::current_token_num].
//!
//! If you need to frequently try different filling values and re-count tokens, you can use [PartialPrompt::with_counter_cache] to get a [PromptTokenCountCache] that can be used to count the number of tokens in the prompt.
//! It's very useful when the template is very long and thus takes a long time to count the number of tokens.
//!
//!


use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::prompt::errors::{PlaceholderNotExist, UnfilledPlaceholders};
use crate::utils::prompt_processing::{get_placeholders, replace_all_placeholders};
use crate::utils::token::{CountToken, PromptTokenCountCache};
use log::warn;
use crate::utils::JsonMap;


/// A prompt template with some placeholders filled. A partial prompt can be only constructed from a prompt template via [PromptTemplate::construct_prompt].
#[derive(Debug, Clone)]
#[readonly::make]
pub struct PartialPrompt {
    /// The template of the partial prompt, readonly
    #[readonly]
    pub template: PromptTemplate,

    /// Mapping from placeholder name to its filling value
    pub(crate) placeholder_to_vals: HashMap<String, Option<String>>,

    /// Record the placeholders that are not filled yet
    pub(crate) unfilled_placeholders: HashSet<String>,
}

impl PartialPrompt {
    /// Fill the placeholders in the partial prompt with the given values.
    /// Panics if the placeholder does not exist.
    pub fn fill(&mut self, placeholder: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.try_fill(placeholder, value).unwrap()
    }

    /// Fill the placeholders in the partial prompt with the given values.
    /// Returns an error if the placeholder does not exist.
    pub fn try_fill(&mut self, placeholder: impl Into<String>, value: impl Into<String>) -> Result<&mut Self, PlaceholderNotExist> {
        let placeholder = placeholder.into();
        if self.placeholder_to_vals.contains_key(&placeholder) {
            self.unfilled_placeholders.remove(&placeholder);
            self.placeholder_to_vals.insert(placeholder, Some(value.into()));
            Ok(self)
        } else {
            Err(PlaceholderNotExist::new(placeholder, value, &self.template.placeholders))
        }
    }

    /// Get a [PromptTokenCountCache] that can be used to quickly count the number of tokens in the prompt and cache.
    pub fn with_counter_cache<'a, C: CountToken>(&'a self, counter: &'a C) -> PromptTokenCountCache<'a, C> {
        PromptTokenCountCache::new(self, counter)
    }

    /// Count the number of tokens in the prompt without caching. Note that the unfilled placeholders are also counted with the placeholder names.
    pub fn current_token_num(&self, counter: &impl CountToken) -> usize {
        let mapping: HashMap<String, String> = self.placeholder_to_vals.iter().filter_map(|(p, v)| {
            v.as_ref().and_then(|v| Some((p.clone(), v.clone())))
        }).collect();
        PromptTokenCountCache::new(self, counter).attempt_fill_multiple_and_count(&mapping).unwrap()
    }

    /// Complete the partial prompt and return the completed prompt.
    /// Returns an error if there are still unfilled placeholders.
    pub fn complete(&self) -> Result<String, UnfilledPlaceholders> {
        if self.unfilled_placeholders.is_empty() {
            let template = self.template.str();
            let prompt = unsafe { replace_all_placeholders(template, &self.placeholder_to_vals) };
            Ok(prompt)
        } else {
            Err(UnfilledPlaceholders {
                all_placeholders: self.template.placeholders.iter().map(Clone::clone).collect(),
                unfilled_placeholders: self.unfilled_placeholders.iter().map(|s| (*s).clone()).collect(),
            })
        }
    }
}

/// A prompt template with placeholders. It can also have metadata in JSON format.
#[derive(Debug, Clone)]
#[readonly::make]
pub struct PromptTemplate {
    /// The template of the partial prompt, immutable
    template: Arc<String>,

    /// The placeholders in the template, readonly
    #[readonly]
    pub placeholders: HashSet<String>,

    /// The metadata of the prompt template, readonly
    #[readonly]
    pub meta_data: Arc<JsonMap>,
}

impl PromptTemplate {
    /// Create a prompt template from a string without metadata.
    pub fn new(template: impl Into<String>) -> Self {
        Self::with_metadata(template, JsonMap::new())
    }

    /// Create a prompt template from a string with metadata. Warns if the template does not have any placeholder.
    pub fn with_metadata(template: impl Into<String>, metadata: JsonMap) -> Self {
        let template = template.into();
        let placeholders = get_placeholders(&template);
        if placeholders.len() == 0 {
            warn!("Your prompt template does not have a placeholder. If this is intended, ignore this message. \
            Otherwise, check whether you have written placeholders correctly.\n\
            Got prompt template:\n\
            {}", template);
        }
        Self {
            template: Arc::new(template),
            meta_data: Arc::new(metadata),
            placeholders,
        }
    }

    /// Get the prompt template as a string.
    #[inline]
    pub fn str(&self) -> &str {
        &self.template
    }

    /// Construct a partial prompt from the prompt template.
    pub fn construct_prompt(&self) -> PartialPrompt {
        PartialPrompt {
            template: self.clone(),
            placeholder_to_vals: self.placeholders.iter().map(|p| (p.clone(), None)).collect(),
            unfilled_placeholders: self.placeholders.clone(),
        }
    }
}

pub mod errors {
    use std::collections::HashSet;
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;

    /// Error when trying to complete a partial prompt but there are still unfilled placeholders.
    #[derive(Debug)]
    pub struct UnfilledPlaceholders {
        pub unfilled_placeholders: Vec<String>,
        pub all_placeholders: Vec<String>,
    }

    impl fmt::Display for UnfilledPlaceholders {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "UnfilledPlaceholders: to complete the prompt template,\n  Requires Placeholders:{:?}\n  Unfilled Placeholders:{:?}",
                   self.all_placeholders, self.unfilled_placeholders)
        }
    }

    impl Error for UnfilledPlaceholders {}

    /// Error when trying to fill a placeholder that does not exist in the prompt template of the partial prompt.
    #[derive(Debug)]
    pub struct PlaceholderNotExist {
        pub try_fill_placeholder: String,
        pub value: String,
        pub available_placeholders: Vec<String>,
    }

    impl PlaceholderNotExist {
        pub(crate) fn new(try_fill_placeholder: impl Into<String>,
                          value: impl Into<String>,
                          available_placeholders: &HashSet<String>) -> Self {
            let available_placeholders = available_placeholders.iter().map(|k| k.clone()).collect();
            PlaceholderNotExist {
                try_fill_placeholder: try_fill_placeholder.into(),
                value: value.into(),
                available_placeholders,
            }
        }
    }

    impl fmt::Display for PlaceholderNotExist {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "PlaceholderNotExist: try to fill placeholder = {} with value = {}, but available placeholders are {:?}",
                   self.try_fill_placeholder,
                   self.value,
                   self.available_placeholders)
        }
    }

    impl Error for PlaceholderNotExist {}
}

#[cfg(test)]
mod test_prompt {
    // TODO
}



