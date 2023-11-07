use crate::models::*;
use {
    serde::{Serialize,Deserialize},
    std::collections::HashMap,
    super::req_and_res,
    req_and_res::{ChatCompletionMessage}, gpt_models::GptModel
};



#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: req_and_res::Usage,
}

impl ChatCompletionResponse {
    
    /// Return cost in CENTS given the model used
    pub fn cost(&self, model: &GptModel) -> f32 {
        //usage: &Usage, model: &GptModel
        let usage = &self.usage;
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

        { (
            ( usage.prompt_tokens as f32  * prompt_cost_factor/ 1000.0 ) 
            + 
            ( usage.completion_tokens as f32 * completion_cost_factor/1000.0 )
        ) as f32 * 100.0} // As cents 

    }
}

// impl ChatCompletionResponse {
//     fn default(content: String, model: GptModel) -> Self {
//         Self { 
//             created: 0 ,
//             id: "All response fields defaulted excet.".to_string(), 
//             choices: vec![ChatCompletionChoice {finish_reason: FinishReason::stop, index: 0, message: content}],
//             model: model.to_string(),
//             object: "chat.completion".to_string(),
//             usage: Usage {completion_tokens: 0, prompt_tokens: 0, total_tokens}
//         }
//     }
// }


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatCompletionChoice {
    pub index: i64,
    pub message: ChatCompletionMessage,
    pub finish_reason: FinishReason,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub schema_type: JSONSchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<JSONSchemaDefine>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JSONSchemaType {
    Object,
    Number,
    String,
    Array,
    Null,
    Boolean,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONSchemaDefine {
    #[serde(rename = "type")]
    pub schema_type: Option<JSONSchemaType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<JSONSchemaDefine>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JSONSchemaDefine>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub enum FinishReason {
    stop,
    length,
    function_call,
    content_filter,
    null,
}

