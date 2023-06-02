//! # Fillers
//!
//! Fillers are used to fill the placeholders in the prompt template.

use anyhow::Result;

use crate::prompt::PartialPrompt;

/// Used to get the placeholders to fill.
pub trait FillPlaceholders {
    /// Return the placeholders to fill.
    fn placeholders_to_fill(&self) -> &Vec<String>;
}

/// Used to fill the placeholders in the prompt template.
/// Returns errors if the placeholders are not filled.
pub trait Fill: FillPlaceholders {
    fn fill(&self, partial_prompt: &mut PartialPrompt) -> Result<()>;
}

/// Used to fill the placeholders in the prompt template.
/// Returns errors if the placeholders are not filled.
/// Mut version of Fill.
pub trait FillMut: FillPlaceholders {
    fn fill_mut(&mut self, partial_prompt: &mut PartialPrompt) -> Result<()>;
}

/// Used to fill the placeholders in the prompt template with any context.
/// Returns errors if the placeholders are not filled, else a possibly new context.
pub trait FillWith<CTX>: FillPlaceholders {
    fn fill_with(&self, partial_prompt: &mut PartialPrompt, context: CTX) -> Result<CTX>;
}

/// Used to fill the placeholders in the prompt template with any context.
/// Returns errors if the placeholders are not filled, else a possibly new context.
/// Mut version of FillWith.
pub trait FillWithMut<CTX>: FillPlaceholders {
    fn fill_with_mut(&mut self, partial_prompt: &mut PartialPrompt, context: CTX) -> Result<CTX>;
}

/// blanket Fill impl for FillWith<()> because it requires no context.
impl<T: FillWith<()>> Fill for T {
    fn fill(&self, partial_prompt: &mut PartialPrompt) -> Result<()> {
        self.fill_with(partial_prompt, ())
    }
}

/// blanket FillMut impl for FillWithMut<()> because it requires no context.
impl<T: FillWithMut<()>> FillMut for T {
    fn fill_mut(&mut self, partial_prompt: &mut PartialPrompt) -> Result<()> {
        self.fill_with_mut(partial_prompt, ())
    }
}