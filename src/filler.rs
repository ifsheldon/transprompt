use crate::prompt::PartialPrompt;
use anyhow::Result;

pub trait FillPlaceholders {
    fn placeholders_to_fill(&self) -> &Vec<String>;
}

pub trait Fill: FillPlaceholders {
    fn fill(&self, partial_prompt: &mut PartialPrompt) -> Result<()>;
}

pub trait FillMut: FillPlaceholders {
    fn fill_mut(&mut self, partial_prompt: &mut PartialPrompt) -> Result<()>;
}

pub trait FillWith<CTX>: FillPlaceholders {
    fn fill_with(&self, partial_prompt: &mut PartialPrompt, context: CTX) -> Result<CTX>;
}

pub trait FillWithMut<CTX>: FillPlaceholders {
    fn fill_with_mut(&mut self, partial_prompt: &mut PartialPrompt, context: CTX) -> Result<CTX>;
}

impl<T: FillWith<()>> Fill for T {
    fn fill(&self, partial_prompt: &mut PartialPrompt) -> Result<()> {
        self.fill_with(partial_prompt, ())
    }
}

impl<T: FillWithMut<()>> FillMut for T {
    fn fill_mut(&mut self, partial_prompt: &mut PartialPrompt) -> Result<()> {
        self.fill_with_mut(partial_prompt, ())
    }
}