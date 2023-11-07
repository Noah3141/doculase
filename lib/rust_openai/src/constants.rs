/// These constants encode the cost in cents/1000 tokens of prompts, and completions, by model.
pub mod cost_factors {
    use crate::models::gpt_models::GptModel;
    use crate::models::req_and_res::Usage;

    /// Returns the cost in cents/1000 tokens for a given model
    pub fn compute_cost(usage: &Usage, model: &GptModel) -> f32 {
        
        let (prompt_cost_factor, completion_cost_factor) = match model {
            GptModel::Gpt35Turbo => (0.0015, 0.002), //4k
            GptModel::Gpt35Turbo16k => (0.003, 0.004), //16k

            GptModel::Gpt35Turbo0613 => (0.0015, 0.002), // 4k

            GptModel::Gpt4 => (0.03, 0.06),
            GptModel::Gpt40314 => (0.03, 0.06),
            GptModel::Gpt40613 => (0.03, 0.06),

            GptModel::Gpt432k => (0.06, 0.12),
            GptModel::Gpt432k0314 => (0.06, 0.12),
        };

        {
            ( usage.prompt_tokens as f32 * prompt_cost_factor / 1000.0) // cost per 1000 / 1000 = cost per 1 => cost per 1 * tokens = cost of tokens
            + 
            ( usage.completion_tokens as f32 * completion_cost_factor / 1000.0 )
        }

    }

}

/// These constants encode the strings use to refer to each model in the official OpenAI docs
pub mod model_strings {
    pub const GPT3_5_TURBO: &str = "gpt-3.5-turbo"; // 4k
    pub const GPT3_5_TURBO_0613: &str = "gpt-3.5-turbo-0613"; 
    
    pub const GPT3_5_TURBO_16K: &str = "gpt-3.5-turbo-16k"; // 16k

    pub const GPT4: &str = "gpt-4"; // 16k
    pub const GPT4_0314: &str = "gpt-4-0314"; 
    pub const GPT4_0613: &str = "gpt-4-0613";
    
    pub const GPT4_32K: &str = "gpt-4-32k"; // 32k
    pub const GPT4_32K_0314: &str = "gpt-4-32k-0314";
}



pub mod pdf_path {
    pub const DEFAULT_PDF_DIR: &str = "./pdfs/";

}