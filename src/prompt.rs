use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serde_json::{Map, Value};
use crate::prompt::errors::{PlaceholderNotExist, UnfilledPlaceholders};
use crate::utils::{get_placeholders, replace_all_placeholders};
use log::warn;

pub type JsonMap = Map<String, Value>;


#[derive(Debug, Clone)]
pub struct PartialPrompt {
    pub template: PromptTemplate,
    pub(crate) placeholder_to_vals: HashMap<String, Option<String>>,
    pub(crate) unfilled_placeholders: HashSet<String>,
}

impl PartialPrompt {
    pub fn fill<S: Into<String>>(&mut self, placeholder: S, value: S) -> &mut Self {
        self.try_fill(placeholder, value).unwrap()
    }

    pub fn try_fill<S: Into<String>>(&mut self, placeholder: S, value: S) -> Result<&mut Self, PlaceholderNotExist> {
        let placeholder = placeholder.into();
        if self.placeholder_to_vals.contains_key(&placeholder) {
            self.unfilled_placeholders.remove(&placeholder);
            self.placeholder_to_vals.insert(placeholder, Some(value.into()));
            Ok(self)
        } else {
            Err(PlaceholderNotExist::new(placeholder, value.into(), self.placeholder_to_vals.keys()))
        }
    }

    pub fn finish(&self) -> Result<String, UnfilledPlaceholders> {
        if self.unfilled_placeholders.is_empty() {
            let template = self.template.str();
            let prompt = unsafe { replace_all_placeholders(template, &self.placeholder_to_vals) };
            Ok(prompt)
        } else {
            Err(UnfilledPlaceholders {
                all_placeholders: self.template.placeholders().iter().map(Clone::clone).collect(),
                unfilled_placeholders: self.unfilled_placeholders.iter().map(|s| (*s).clone()).collect(),
            })
        }
    }
}


#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub meta_data: Arc<JsonMap>,
    template: Arc<String>,
    placeholders: HashSet<String>,
}

impl PromptTemplate {
    pub fn new(template: String) -> Self {
        Self::with_metadata(template, JsonMap::new())
    }

    pub fn with_metadata(template: String, metadata: JsonMap) -> Self {
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

    #[inline]
    pub fn placeholders(&self) -> &HashSet<String> {
        &self.placeholders
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
    use std::collections::hash_map::Keys;
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
        pub(crate) fn new(try_fill_placeholder: String,
                          value: String,
                          available_placeholders: Keys<'_, String, Option<String>>) -> Self {
            let available_placeholders = available_placeholders.map(|k| k.clone()).collect();
            PlaceholderNotExist {
                try_fill_placeholder,
                value,
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



