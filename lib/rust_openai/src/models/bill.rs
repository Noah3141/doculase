use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bill {
    /// Total cost so far since last `.reset_bill()` in CENTS
    pub cost: f32,
    /// Total number of prompt tokens (used according to size of prompts) used so far since last `.reset_bill()`
    pub prompt_tokens: i32,
    /// Total number of completion tokens (used according to size of responses) so far since last `.reset_bill()`
    pub completion_tokens: i32,
    /// Total token usage so far since last `.reset_bill()`
    pub total_tokens: i32,
    /// Number of queries recorded so far since last `.reset_bill()`
    pub query_count: i32,
    /// Number of times a ChatGPT completion was pulled from the cache instead of the API, because the prompt was found in the cache
    pub cache_retrievals: i32,
}

impl Default for Bill {
    fn default() -> Bill {
        Bill {
            cache_retrievals: 0,
            completion_tokens: 0,
            prompt_tokens: 0,
            cost: 0.00,
            query_count: 0,
            total_tokens: 0
        }
    }
}