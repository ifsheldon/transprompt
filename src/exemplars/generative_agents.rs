use crate::filler::{FillPlaceholders, FillWithMut};
use crate::prompt::{PartialPrompt, PromptTemplate};
use crate::utils::timing::VirtualTime;
use crate::utils::vec_stores::QdrantCloudDB;

pub struct GAConfig {
    recency_decay_factor: f32,
    recency_weight: f32,
    importance_weight: f32,
    relevance_weight: f32,
    time_origin: VirtualTime,
    name: String,
    innate_trait_description: String,
}

impl Default for GAConfig {
    fn default() -> Self {
        Self {
            recency_decay_factor: 0.9,
            recency_weight: 1.0,
            importance_weight: 1.0,
            relevance_weight: 1.0,
            time_origin: VirtualTime::default(),
            name: "assistant".to_string(),
            innate_trait_description: "helpful and friendly".to_string(),
        }
    }
}

pub struct GenerativeAgent {
    config: GAConfig,
    general_response_template: PromptTemplate,
    dialogue_response_template: PromptTemplate,
    agent_status: String,
    memory: GAMemory,
}

impl GenerativeAgent {
    const DIALOGUE_RESPONSE_TEMPLATE_STR: &'static str = r#"{{agent_summary_description}}
It is {{current_time}}.
{{agent_name}}'s status: {{agent_status}}
Summary of relevant context from {{agent_name}}'s memory:
{{relevant_memories}}
Most recent observations: {{most_recent_memories}}
Observation: {{observation}}
What would {{agent_name}} say? To end the conversation, write:
GOODBYE: "what to say".
Otherwise to continue the conversation,write:
SAY: "what to say next""#;

    const GENERAL_RESPONSE_TEMPLATE_STR: &'static str = r#"{{agent_summary_description}}
It is {{current_time}}.
{{agent_name}}'s status: {{agent_status}}
Summary of relevant context from {{agent_name}}'s memory:
{{relevant_memories}}
Most recent observations: {{most_recent_memories}}
Observation: {{observation}}
Should {{agent_name}} react to the observation, and if so, what would be an appropriate reaction? Respond in one line. If the action is to engage in dialogue, write:
SAY: "what to say"
otherwise, write:
REACT: {agent_name}'s reaction (if anything).
Either do nothing, react, or say something but not both."#;


    pub fn new(config: GAConfig) -> Self {
        let dialogue_response_template = PromptTemplate::new(Self::DIALOGUE_RESPONSE_TEMPLATE_STR);
        let general_response_template = PromptTemplate::new(Self::GENERAL_RESPONSE_TEMPLATE_STR);
        todo!()
    }

    pub fn get_importance(&self, event: &str) -> u8 {
        todo!()
    }
}


pub struct GAContext {
    pub time_origin: VirtualTime,
    pub now: VirtualTime,
    pub event: String,
    pub topk_relevant_memory: usize,
}

pub struct GAMemory {
    pub recency_decay_factor: f32,
    pub recency_weight: f32,
    pub importance_weight: f32,
    pub relevance_weight: f32,
    pub time_origin: VirtualTime,
    database: QdrantCloudDB,
    placeholders_to_fill: Vec<String>,
}

impl GAMemory {
    const ASK_IMPORTANCE_TEMPLATE_STR: &'static str = r#"On the scale of 1 to 10, where 1 is purely mundane (e.g., brushing teeth, making bed) and 10 is extremely poignant (e.g., a break up, college acceptance), rate the likely poignancy of the following piece of memory. Respond with a single integer.
Memory: ```{{memory_content}}```
Rating: "#;

    const RELEVANT_MEMORY_PLACEHOLDER: &'static str = "relevant_memories";
    const MOST_RECENT_MEMORIES_PLACEHOLDER: &'static str = "most_recent_memories";

    pub fn new(recency_decay_factor: f32, recency_weight: f32, importance_weight: f32, relevance_weight: f32, time_origin: VirtualTime, database: QdrantCloudDB) -> Self {
        Self {
            recency_decay_factor,
            recency_weight,
            importance_weight,
            relevance_weight,
            time_origin,
            database,
            placeholders_to_fill: vec![Self::RELEVANT_MEMORY_PLACEHOLDER.to_string(), Self::MOST_RECENT_MEMORIES_PLACEHOLDER.to_string()],
        }
    }

    pub fn find_relevant_memory(&self, event: &str, topk: usize) -> Vec<String> {
        todo!()
    }
}


impl FillPlaceholders for GAMemory {
    fn placeholders_to_fill(&self) -> &Vec<String> {
        &self.placeholders_to_fill
    }
}

impl FillWithMut<GAContext> for GAMemory {
    fn fill_with_mut(&mut self, partial_prompt: &mut PartialPrompt, context: GAContext) -> anyhow::Result<GAContext> {
        let relevant_memories = self.find_relevant_memory(context.event.as_str(), context.topk_relevant_memory);
        // TODO: find most recent memories given a context length budget
        partial_prompt
            .try_fill(Self::RELEVANT_MEMORY_PLACEHOLDER, relevant_memories.join("\n"))?
            .try_fill(Self::MOST_RECENT_MEMORIES_PLACEHOLDER, "")?;
        Ok(context)
    }
}

#[cfg(test)]
mod test {
    use crate::exemplars::generative_agents::GenerativeAgent;

    #[test]
    fn test_print() {
        println!("{}", GenerativeAgent::GENERAL_RESPONSE_TEMPLATE_STR)
    }
}