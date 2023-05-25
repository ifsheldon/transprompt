use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::prompt::errors::{PlaceholderNotExist, UnfilledPlaceholders};
use crate::utils::prompt_processing::{get_placeholders, replace_all_placeholders};
use crate::utils::token::{CountToken, PromptTokenCountCache};
use log::warn;
use crate::utils::JsonMap;


#[derive(Debug, Clone)]
#[readonly::make]
pub struct PartialPrompt {
    #[readonly]
    pub template: PromptTemplate,
    pub(crate) placeholder_to_vals: HashMap<String, Option<String>>,
    pub(crate) unfilled_placeholders: HashSet<String>,
}

impl PartialPrompt {
    pub fn fill(&mut self, placeholder: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.try_fill(placeholder, value).unwrap()
    }

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

    pub fn with_counter_cache<'a, C: CountToken>(&'a self, counter: &'a C) -> PromptTokenCountCache<'a, C> {
        PromptTokenCountCache::new(self, counter)
    }

    pub fn current_token_num(&self, counter: &impl CountToken) -> usize {
        let mapping: HashMap<String, String> = self.placeholder_to_vals.iter().filter_map(|(p, v)| {
            v.as_ref().and_then(|v| Some((p.clone(), v.clone())))
        }).collect();
        PromptTokenCountCache::new(self, counter).attempt_fill_multiple_and_count(&mapping).unwrap()
    }

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


#[derive(Debug, Clone)]
#[readonly::make]
pub struct PromptTemplate {
    template: Arc<String>,
    #[readonly]
    pub placeholders: HashSet<String>,
    #[readonly]
    pub meta_data: Arc<JsonMap>,
}

impl PromptTemplate {
    pub fn new(template: impl Into<String>) -> Self {
        Self::with_metadata(template, JsonMap::new())
    }

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

    #[inline]
    pub fn str(&self) -> &str {
        &self.template
    }

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



