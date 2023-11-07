use super::query_cache::Model;
use crate::models::*;


impl Model {
    pub fn to_query(&self) -> Query {
        
        Query { 
            prompt: self.prompt.clone(), 
            cost: self.cost, 
            response: serde_json::from_value(self.response.clone()).unwrap(), 
            process_time: self.process_time as u64, 
            model: GptModel::from_string(&self.model), 
            query_type: if self.query_key == self.prompt {QueryType::PromptCompletion } else if self.query_key.contains("Meta-Battery") {QueryType::MetaCompletion} else {QueryType::PdfCompletion}, 
            temperature: self.temperature,
            from_cache: true, 
        }

    }

}