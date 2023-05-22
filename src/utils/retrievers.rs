pub mod generative_agents {
    use crate::utils::vec_stores::QdrantCloudDB;
    use chrono::prelude::*;
    use crate::filler::{FillPlaceholders, FillWithMut};
    use crate::prompt::{PartialPrompt, PromptTemplate};

    pub type VirtualTime = NaiveDateTime;

    pub fn create_virtual_time(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<VirtualTime> {
        NaiveDate::from_ymd_opt(year as i32, month, day)
            .and_then(|date| date.and_hms_opt(hour, min, sec))
    }


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
        response_template: PromptTemplate,
        response_template_filler: GAResponseTemplateFiller,
        ask_importance_template: PromptTemplate,
    }

    impl GenerativeAgent {
        pub fn new(config: GAConfig) -> Self {
            todo!()
        }

        pub fn get_importance(&self, event: &str) -> u8 {
            let prompt = self.ask_importance_template
                .construct_prompt()
                .fill("event", event)
                .finish()
                .unwrap();
            todo!()
        }
    }


    pub struct GAContext {
        pub time_origin: VirtualTime,
        pub now: VirtualTime,
        pub event: String,
    }

    pub struct GAResponseTemplateFiller {
        recency_decay_factor: f32,
        recency_weight: f32,
        importance_weight: f32,
        relevance_weight: f32,
        database: QdrantCloudDB,
        placeholders_to_fill: Vec<String>,
        time_origin: VirtualTime,
    }

    impl GAResponseTemplateFiller {
        pub fn new() -> Self {
            todo!()
        }
    }


    impl FillPlaceholders for GAResponseTemplateFiller {
        fn placeholders_to_fill(&self) -> &Vec<String> {
            &self.placeholders_to_fill
        }
    }

    impl FillWithMut<GAContext> for GAResponseTemplateFiller {
        fn fill_with_mut(&mut self, partial_prompt: &mut PartialPrompt, context: GAContext) -> anyhow::Result<GAContext> {
            todo!()
        }
    }
}

#[cfg(test)]
mod test_retrievers {
    use crate::utils::retrievers::generative_agents::create_virtual_time;

    #[test]
    fn test_virtual_time() {
        let before = create_virtual_time(2006, 7, 8, 9, 10, 11).unwrap();
        let after = create_virtual_time(2006, 8, 8, 9, 10, 11).unwrap();
        let duration = after - before;
        assert_eq!(duration.num_days(), 31);
    }
}

