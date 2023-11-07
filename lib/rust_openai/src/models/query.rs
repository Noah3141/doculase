use serde::{Serialize, Deserialize};

use crate::GptModel;

use super::ChatCompletionResponse;


/// An individual Query, representing a prompt-completion event, and its metadata <br>
/// A query can hold a Summary event for a pdf, in which case its query_type will be PdfCompletion, and its prompt field will be a stamp of the corresponding battery used to generate the summary. See `Battery`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query {
    /// The prompt that was sent for chat completion if QueryType is chat completion, else this field is field with a stamp corresponding to the question battery that was used
    pub prompt: String,
    /// The cost of the request-response interaction (both prompt and completion tokens)
    pub cost: f32,
    /// The response given to this query's prompt field. `response` holds the metrics which are tracked in a running total in this OpenAIAccount's `bill`
    pub response: ChatCompletionResponse,
    /// The time it took from sending this Query's prompt, to receiving this Query's response.
    pub process_time: u64,
    /// Model used to generate the response
    pub model: GptModel,
    /// The key in the cache for a prompt completion is the prompt, whereas the key for a PdfCompletion is the pdf's filename, which should always match its storage name on disc, plus a stamp corresponding to the battery used upon the pdf for the completion.
    /// 
    pub query_type: QueryType,
    pub temperature: f32,
    pub from_cache: bool,
}

impl Query {
    pub fn response(&self) -> &ChatCompletionResponse {
        &self.response
    }
}


/// The type of request response that occured for this query. A prompt completion involved Chat Completion from a prompt, whereas a PDF summary is generated from PDF. <br>
/// The key in the cache for a prompt completion is the prompt, whereas the key for a PdfCompletion is the pdf's filename, which should always match its storage name on disc.
#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub enum QueryType {
    PromptCompletion,
    PdfCompletion,
    MetaCompletion
}