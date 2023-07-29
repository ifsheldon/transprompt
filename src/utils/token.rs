//! Token counting traits and utilities

use std::collections::{HashMap, HashSet};

use crate::prompt::errors::PlaceholderNotExist;
use crate::prompt::PartialPrompt;
use crate::utils::prompt_processing::{PLACEHOLDER_MATCH_RE, strip_format};

pub mod tiktoken;

/// Trait for counting tokens in a string.
pub trait CountToken {
    fn count_token(&self, string: &str) -> usize;
}

/// Blanket impl of CountToken for Fn(&str) -> usize.
impl<F> CountToken for F where F: Fn(&str) -> usize {
    fn count_token(&self, string: &str) -> usize {
        self(string)
    }
}

/// Count the number of tokens in a string by the length of the string.
#[inline]
pub fn count_tokens_by_len(string: &str) -> usize {
    string.len()
}


/// Cache for counting tokens in a [PartialPrompt](crate::prompt::PartialPrompt).
#[derive(Debug, Clone)]
#[readonly::make]
pub struct PromptTokenCountCache<'a, C: CountToken> {
    /// The token count of the template of the partial prompt. Note that placeholders are also counted with the placeholder names.
    #[readonly]
    pub template_token_count: usize,
    all_placeholders: &'a HashSet<String>,
    placeholder_to_val: &'a HashMap<String, Option<String>>,
    placeholder_occurrence: HashMap<&'a str, usize>,
    placeholder_token_count: HashMap<&'a str, usize>,
    counter: &'a C,
}

impl<'a, C: CountToken> PromptTokenCountCache<'a, C> {
    fn get_placeholder_occurrence(string: &'a str, placeholders: &'a HashSet<String>) -> HashMap<&'a str, usize> {
        let mut count: HashMap<&str, usize> = placeholders.into_iter().map(|s| (s.as_str(), 0)).collect();
        PLACEHOLDER_MATCH_RE
            .captures_iter(string)
            .for_each(|captures| {
                let placeholder_name = strip_format(&captures[0]);
                let count = count.get_mut(placeholder_name).unwrap();
                *count += 1;
            });
        count
    }

    /// Create a new cache for counting tokens in a [PartialPrompt](crate::prompt::PartialPrompt).
    pub fn new(partial_prompt: &'a PartialPrompt, counter: &'a C) -> Self {
        let template_str = partial_prompt.template.str();
        let template_token_count = counter.count_token(template_str);
        let placeholder_occurrence = Self::get_placeholder_occurrence(template_str, &partial_prompt.template.placeholders);
        let placeholder_token_count = partial_prompt.template.placeholders.iter().map(|p| (p.as_str(), counter.count_token(p))).collect();
        Self {
            template_token_count,
            all_placeholders: &partial_prompt.template.placeholders,
            placeholder_to_val: &partial_prompt.placeholder_to_vals,
            placeholder_occurrence,
            placeholder_token_count,
            counter,
        }
    }

    /// Count the number of tokens in a [PartialPrompt](crate::prompt::PartialPrompt) with the placeholder filled with the given value.
    /// Note that this does not change the partial prompt itself. Unfilled placeholders are also counted with the placeholder names.
    /// Returns an error if the placeholder does not exist.
    pub fn attempt_fill_and_count(&self, placeholder_name: impl Into<String>, fill_value: impl Into<String>) -> Result<usize, PlaceholderNotExist> {
        let placeholder_name = placeholder_name.into();
        let fill_value = fill_value.into();
        if self.placeholder_occurrence.contains_key(placeholder_name.as_str()) {
            let old_count = self.template_token_count;
            let total_delta: usize = self.all_placeholders.iter()
                .map(|placeholder| {
                    let placeholder = placeholder.as_str();
                    let fill_value = if placeholder == placeholder_name {
                        Some(&fill_value)
                    } else {
                        self.placeholder_to_val.get(placeholder).unwrap().as_ref()
                    };
                    let fill_value_token_count = fill_value.map_or(0, |s| self.counter.count_token(s));
                    let placeholder_token_count = *self.placeholder_token_count.get(placeholder).unwrap();
                    let placeholder_occurrence = *self.placeholder_occurrence.get(placeholder).unwrap();
                    let delta = (fill_value_token_count - placeholder_token_count) * placeholder_occurrence;
                    delta
                })
                .sum();

            Ok(old_count + total_delta)
        } else {
            Err(PlaceholderNotExist::new(placeholder_name, fill_value, self.all_placeholders))
        }
    }

    /// Count the number of tokens in a [PartialPrompt](crate::prompt::PartialPrompt) with the placeholders filled with the given values.
    /// Note that this does not change the partial prompt itself. Unfilled placeholders are also counted with the placeholder names.
    /// Returns an error if any of the placeholders does not exist.
    pub fn attempt_fill_multiple_and_count(&self, mappings: &HashMap<String, String>) -> Result<usize, PlaceholderNotExist> {
        for (placeholder_to_fill, value) in mappings {
            if !self.all_placeholders.contains(placeholder_to_fill.as_str()) {
                return Err(PlaceholderNotExist::new(placeholder_to_fill, value, &self.all_placeholders));
            }
        }
        let old_count = self.template_token_count;
        let total_delta: usize = self.all_placeholders.iter()
            .map(|placeholder| {
                let placeholder = placeholder.as_str();
                let fill_value = mappings.get(placeholder).or(self.placeholder_to_val.get(placeholder).unwrap().as_ref());
                let fill_value_token_count = fill_value.map_or(0, |s| self.counter.count_token(s));
                let placeholder_token_count = *self.placeholder_token_count.get(placeholder).unwrap();
                let placeholder_occurrence = *self.placeholder_occurrence.get(placeholder).unwrap();
                let delta = (fill_value_token_count - placeholder_token_count) * placeholder_occurrence;
                delta
            })
            .sum();

        Ok(old_count + total_delta)
    }
}

#[cfg(test)]
mod test_token {
    use super::CountToken;

    #[test]
    fn test_str_len_impl() {
        let counter = str::len;
        let size = counter.count_token("");
        assert_eq!(0, size);
    }
}