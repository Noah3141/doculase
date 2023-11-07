use reqwest::Response;
use sea_orm::{DatabaseConnection, Database, EntityTrait, QueryFilter, ColumnTrait};
use std::{collections::HashMap, env, };
use crate::constants::pdf_path::DEFAULT_PDF_DIR;
use crate::models::api_error::APIError;

use crate::models::hash::calculate_hash;
use crate::models::{*};
use crate::{*};

use std::fs;
use std::io;

const API_URL_V1: &str = "https://api.openai.com/v1";

pub const BILL_FILEPATH: &str = "bill.json";
pub const CACHE_FILEPATH: &str = "cache.json"; // "./src/research_sets/../cache.json"


#[derive(Clone, Debug)]
pub struct OpenAIAccount  { 
    /// Choose from models::gpt_models From this
    model: GptModel,
    /// Default value looks for `CHATGPT_API_KEY` environment var
    api_key: String,
    /// `0.0 - 0.4`: Produces more focused, conservative, and consistent responses. <br> `0.5 - 0.7`: Strikes a balance between creativity and consistency. <br> `0.8 - 1.0`: Generates more creative, diverse, and unexpected outputs. <br> Default sets to 0.0
    temperature: f32,
    /// Attribute used to save and retrieve running metrics, which are running totals of Query metrics. 
    /// <br> This variable is serialized into and deserialized from this OpenAIAccount's `.cache_filepath` attribute. The running total can be reset with ...
    /// <br> See struct `Bill` for a list of what is tracked.
    bill: Bill,
    /// Attribute used to save and retrieve Query metrics. 
    /// This variable is serialized into and deserialized from CACHE_FILEPATH constant.
    /// If a query completion is sent, and the prompt is already found in the cache, the cached response is retrieved, and a new API request is not sent.
    /// Keys are prompts, values are Queries (which themselves hold the prompt, model, etc.)
    pub cache: HashMap<String,Query>,
    
}



impl Default for OpenAIAccount {
    fn default() -> OpenAIAccount {
        OpenAIAccount {
            api_key: env::var("CHATGPT_API_KEY").unwrap().to_string(),
            temperature: 0.0,
            cache: HashMap::new(),
            bill: Bill {..Default::default()},
            model: GptModel::Gpt35Turbo16k,
        }
    }
}

/// The atoms of `OpenAIAccount` functionality, such as initiators, getters, setters, etc., for combination in larger functions
impl OpenAIAccount {
    
    /// Create a new instance of the OpenAIAccount, taking a `GptModel`, temperature
    /// <br> `0.0 - 0.4`: Produces more focused, conservative, and consistent responses. <br> `0.5 - 0.7`: Strikes a balance between creativity and consistency. <br> `0.8 - 1.0`: Generates more creative, diverse, and unexpected outputs. <br> Default sets to 0.0
    /// <br> Initializing an `OpenAIAccount` with .new() clears the backup ("graveyard"). Because of this, initializing the client twice within a project endpoint is not recommended, instead, there should be sufficient getters-setters to make adjustments midway through an analysis.
    /// 
    /// # Errors
    /// Assumes access to files at BILL_FILEPATH and CACHE_FILEPATH variables
    /// <br>
    /// <br> 
    pub fn new(model: GptModel, temperature: f32, ) -> OpenAIAccount {
        let api_key = env::var("CHATGPT_API_KEY").unwrap().to_string();
        // Read the bill into memory or else initialize empty
        
        let bill = match fs::File::open(BILL_FILEPATH) {
            Ok(f) => {
                let reader = io::BufReader::new(f);
                // Read the JSON contents of the file as an instance of...
                let bill: Bill = serde_json::from_reader(reader).unwrap_or_else(|e| {println!("🧾 Initializing client with default blank bill due to:  ❌  {e}") ; Bill {..Default::default()}});
                println!("🧾 Bill read from: {BILL_FILEPATH}");
                bill
            },
            Err(_) => {
                fs::File::create(BILL_FILEPATH).expect("Creation of Bill file, after having not found any file");
                println!("🧾 Empty Bill created at: {BILL_FILEPATH}");
                Bill {..Default::default()}
            },
        };

        // Read the cache into memory or else initialize empty
        let cache: HashMap<String, Query> = match fs::File::open(CACHE_FILEPATH) {
            Ok(f) => {
                let reader = io::BufReader::new(f);
                // Read the JSON contents of the file as an instance of...
                let cache: HashMap<String, Query> = serde_json::from_reader(reader).unwrap_or_else(|e| { if let serde_json::error::Category::Eof = e.classify() {HashMap::new()} else { println!("🗳️  Initializing client with blank cache due to:  ❌  {e}") ; HashMap::new()}  });
                println!("🗳️  Cache read from: {CACHE_FILEPATH}");
                cache
            },
            Err(_) => {
                fs::File::create(CACHE_FILEPATH).expect("Creation of Cache file, after having not found any file");
                println!("🗳️  Empty Cache created at: {CACHE_FILEPATH}");
                HashMap::new()
            },
        };

        let _graveyard = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open("graveyard.json").expect("access to graveyard file");
        println!("🪦  Graveyard backups cleared.");

        println!("🌡️  Model initialized at temperature {temperature}");
        OpenAIAccount {
            bill,
            cache,
            model,
            temperature,
            api_key,
            ..Default::default()
        }
    }

    /// Sends the prompt as the first message, and returns the chat completion response.
    /// <br> Checks cache for presence of prompt, and returns the cache value if present instead of repeating request.
    /// <br> Inputting a model will use that model, otherwise `None` will default to the model used in the .new() initiator.
    pub async fn get_completion(&mut self, prompt: String, model: Option<GptModel>) -> Result<Query, Status> {

        let model = match model {Some(m) => m, None => self.model};

        let query = match self.check_cache(&prompt, QueryType::PromptCompletion) {
            // If found in cache, retrieve the query
            Some(query) => {
                let mut query = query.clone(); 
                query.from_cache = true;
                self.bill.cache_retrievals += 1; 
                self.update_bill(None); 
                println!("--[Cached Answer]--");
                query
            },
            // If absent, send to OpenAI
            None => {
                let from_cache = false;
                let req = ChatCompletionRequest {
                    model: model.to_string(),
                    messages: vec![ChatCompletionMessage {
                        role: MessageRole::user,
                        content: Some(prompt.clone()),
                        name: None,
                        function_call: None,
                    }],
                    functions: None,
                    function_call: None,
                    temperature: Some(self.temperature)
                };

                let start_time = std::time::Instant::now();
                let response = match self.send_completion_request(req).await {Ok(res) => res, Err(e) => return Err(Status::Error(e.to_string()))};
                let process_time = start_time.elapsed().as_secs();

                // Build Query from Response
                let query = Query {prompt: prompt.clone(), response: response.clone(), query_type: QueryType::PromptCompletion, cost: response.cost(&model), process_time, model, temperature: self.temperature, from_cache };
                // Add Query to Cache
                self.cache_query(&prompt, &query);
                // Add data to Bill
                self.update_bill(Some(&query));


                println!("--[Bill so far: ${:.2}]--", self.bill.cost / 100.0);
                println!("--[Took: {}, Cost: ¢{:.4}]--", process_time, (response.cost(&model)));
                query
            },
        };

        Ok(query)


    }

    /// Checks for presence of a Query at the Prompt, returns `Some(Query)` if found in cache, and `None` if absent. 
    /// Converts prompt input to a more uniform format that is used for keys. <br>
    /// - `cache_key` should be either a prompt, to retrive a prompt completion, or a pdf title, to retrieve a summary
    /// - When set to `PromptCompletion`, the cache_key is regularized for whitespace, and lowercased.
    /// - When set to `PdfCompletion`, the cache_key is used as passed, supposedly in title case
    pub fn check_cache(&self, cache_key: &String, query_type: QueryType) -> Option<&Query> {
        // Make the prompt more uniform
        let key = match query_type {
            QueryType::PromptCompletion => cache_key.to_lowercase().replace("\n", " "),
            QueryType::PdfCompletion => cache_key.to_string(),
            QueryType::MetaCompletion => cache_key.to_string(),
        };
        let find = self.cache.get(&key); // if None -> return None
        
        find
    }

    /// Resets both the cache file and in-memory cache to empty
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        match fs::File::create(CACHE_FILEPATH) { Ok(_)=>(), Err(e)=>{println!("\nclear_cache() had trouble initializing a new blank cache file at '{CACHE_FILEPATH}' : \n❌  {e}")}};
        println!("🗳️  Cache cleared at: {CACHE_FILEPATH}");
    }

    pub fn remove_from_cache(&mut self, cache_key: String) -> Option<(String, Query)> {
        let entry = self.cache.remove_entry(&cache_key);
        

        match entry {
            Some(entry) => {
                println!("🗳️  Removed cache entry at key: \"{cache_key}\"");
                // Update the cache file
                let cache_file = match fs::OpenOptions::new().create(true).truncate(true).write(true).open(CACHE_FILEPATH) {Ok(f)=>f, Err(e)=>panic!("🗳️  Could not re-write cache file after removal at {CACHE_FILEPATH}, due to error:  ❌  {e}")};
                serde_json::to_writer_pretty(&cache_file, &self.cache).expect("Serialization of cache to cache file");
                Some(entry)
            },
            None => None
        }
    }

    /// Adds a query to the current in-memory cache, and saves in-memory cache to the cache file <br>
    /// This will overwrite when called outside of a context that has checked the cache with `self.check_cache`
    /// ```
    /// match self.check_cache_for_prompt(&prompt) {
    ///     Some(query) => query,
    ///     None => {
    ///     /* Having found None in cache, make request to OpenAI and process Response into a Query */
    ///     self.cache_query(&prompt, &query);
    ///     }
    /// ``` 
    /// <br>
    /// - Cache key should be the prompt for a PromptCompletion query, or a "{title} - {battery_stamp}" pair for battery based completions.
    pub fn cache_query(&mut self, cache_key: &String, query: &Query) -> () {
        // Make the key uniform if it is a prompt completion
        let cache_key = if let QueryType::PromptCompletion = query.query_type {cache_key.to_lowercase().replace("\n", " ")} else {cache_key.to_string()};
        // Add to self.cache -- checking if something was overwritten, and placing into backup file if so
        match self.cache.insert(cache_key, query.clone()) {None => (), Some(query)=> { 
            let graveyard = std::fs::OpenOptions::new().create(true).append(true).open("graveyard.json").expect("access to graveyard file");
            serde_json::to_writer_pretty(graveyard, &query).expect("Serialization of an overwritten model to the graveyard");
            println!("\n\n");
            println!("🗳️  Caching a query resulted in an overwrite."); 
            println!("🪦  The overwritten query can be found in the graveyard file.");
        }};
        // Save the state of self.cache to file
        let cache = match fs::OpenOptions::new().create(true).truncate(true).write(true).open(CACHE_FILEPATH) {Ok(f)=>f, Err(e)=>panic!("🗳️  Could not cache query at {CACHE_FILEPATH}, due to error:  ❌  {e}")};
        serde_json::to_writer_pretty(&cache, &self.cache).expect("Serialization of cache to cache file");
    }

    pub fn get_bill(&self) -> Bill {
        self.bill.clone()
    }

    /// Bill state is read on ::new(), and stored inside instance. Calling update_bill fully overwrites the bill file.
    /// <br> Passing a query will add that query's usage data to the running bill before writing to file, while passing none will simply write the state of the bill to file. <br>Usually it is called with a Query as the update date, but there are times when one field is alterted directly, and the file is updated to match (cache_retrievals)
    pub fn update_bill(&mut self, query: Option<&Query>) -> () {

        // Take the state of the bill and update it with the data from the Response if a Query was passed
        if let Some(query) = query { 
            let used = query.response.usage.clone();

            // Add to the bill the used amounts
            self.bill.completion_tokens += used.completion_tokens;
            self.bill.prompt_tokens += used.prompt_tokens;
            self.bill.total_tokens += used.total_tokens;
            self.bill.query_count += 1;
            self.bill.cost += query.response.cost(&query.model);
            // self.bill.cache_retrievals
        }

        // Save the state of self.bill to file
        let bill = match fs::OpenOptions::new().create(true).truncate(true).write(true).open(BILL_FILEPATH) {Ok(f)=>f, Err(e)=>panic!("Could not update bill at {BILL_FILEPATH}, due to error:  ❌  {e}")};
        serde_json::to_writer_pretty(&bill, &self.bill).expect("Serialization of bill to bill file");
    }

    /// <br> Fields `completion_tokens`, `prompt_tokens`, `total_tokens`, `query_count`, `cost` are reset.
    /// <br> Field cache_retrievals is left alone
    pub fn reset_bill(&mut self) -> () {
        self.bill.completion_tokens = 0;
        self.bill.prompt_tokens = 0;
        self.bill.total_tokens = 0;
        self.bill.query_count = 0;
        self.bill.cost = 0.00;
        let bill = match fs::OpenOptions::new().create(true).truncate(true).write(true).open(BILL_FILEPATH) {Ok(f)=>f, Err(e)=>panic!("Could not reset bill at {BILL_FILEPATH}, due to error:  ❌  {e}")};
        serde_json::to_writer_pretty(&bill, &self.bill).expect("Serialization of bill to bill file");
        println!("🧾 Bill reset");
    }

    pub fn show_bill(&self) {
        println!("\n");
        println!("🧾 Bill So Far");
        println!("Queries: {}", self.bill.query_count);
        println!("Total Tokens: {}", self.bill.total_tokens);
        println!("Bill: ${:.2}", self.bill.cost / 100.0);
        println!("\n");
    }

    pub fn set_temperature(&mut self, temperature: f32) { 
        if self.temperature < temperature {println!("🌡️  Temperature raised to {temperature}")} else {println!("🌡️  Temperature lowered to {temperature}")}
        self.temperature = temperature; 
    }

}


/// The real methods for asking about a pdf's text
impl OpenAIAccount {

    /// The fully fledged "parse me this pdf please" method. Applies a battery defined in `batteries.rs` to the PDF with the title provided, in the provided directory, saves the response to cache inside a Query stamped with the battery used. <br><br> Here we want the title passed in, so that it can be used for creating the key, and saving the pdf to the provided directory (or the `DEFAULT_PDF_DIR` const if None provided) under `{dir}{title}.pdf`. <br><br>`DEFAULT_PDF_DIR` can be found in `openai_for_rs::constants`
    pub async fn apply_battery_to_pdf(&mut self, pdf_title: String, battery_type: Battery, model: Option<GptModel>, input_dir: Option<String>) -> Result<Query, String> {
        println!("\n--🗳️");
        let dir = match input_dir { None => DEFAULT_PDF_DIR.to_string(), Some(s) => s };
        let model = match model {Some(m) => m, None => self.model};
        let battery_label = battery_type.as_prompt_stamp();
        let path_to_pdf = if dir.ends_with("/") {format!("{dir}{pdf_title}.pdf")} else if dir.contains("\\") {format!("{dir}\\{pdf_title}.pdf")} else {format!("{dir}/{pdf_title}.pdf")};
        let query_key = format!("{pdf_title} - {battery_label}"); 
        
        let query = match self.check_cache(&query_key, QueryType::PdfCompletion) {
            // If found in cache, retrieve the query
            Some(query) => {
                let mut query = query.clone();
                query.from_cache = true; 
                self.bill.cache_retrievals += 1; 
                self.update_bill(None); 
                println!("--[Cached Answer]--");
                query
            },
            // If absent, send to OpenAI
            None => {
                let from_cache = false;
                println!("--[Sending to GPT]--");
                // Load the pdf from the provided file path, or else return to the caller a NotFoundError 
                let pdf = lopdf::Document::load(path_to_pdf).map_err(|e| return e.to_string())?;

                let mut doc = String::new();
                for page in 1..=pdf.get_pages().len() {
                    let content = pdf.extract_text(&[page as u32]).expect("parse");
                    doc.push_str(&content);
                }
                
                let req = ChatCompletionRequest {
                    model: model.to_string(),
                    messages: vec![
                        ChatCompletionMessage {
                            role: MessageRole::user,
                            content: Some(battery_type.to_prompt(doc)),
                            name: None,
                            function_call: None,
                        },
                    ], functions: None, function_call: None, temperature: Some(self.temperature)
                };

                let start_time = std::time::Instant::now();
                let response = match self.send_completion_request(req).await {Ok(res) => res, Err(e) => return Err(e.to_string())};
                let process_time = start_time.elapsed().as_millis() as u64;
                
                println!("--[Completion received]--");

                // Build Query from Response
                let query = Query { prompt: battery_type.as_prompt_stamp(), response: response.clone(), model, process_time, query_type: QueryType::PdfCompletion, cost: response.cost(&model), temperature: self.temperature, from_cache };
                self.cache_query(&query_key, &query); // Add Query to Cache
                self.update_bill(Some(&query)); // Add data to Bill
                println!("--[Bill now shows: ${:.2}]--", self.bill.cost / 100.0);
                println!("--[Took: {}ms, Cost: ¢{:.4}]--", process_time, (query.response.cost(&query.model)));
                query
            },
        };
        println!("--[Got from or created to cache ('./{CACHE_FILEPATH}') under key: \"{pdf_title} - {battery_label}\"]--");
        println!("--");
        Ok(query)
    }

    /// Apply the provided prompt question to a pdf
    pub async fn ask_about_pdf(&mut self, pdf_title: String, prompt: String, model: Option<GptModel>) -> Result<Query, Status> {
        println!("--");
        
        let model = match model {Some(m) => m, None => self.model};
        let prompt = prompt.to_lowercase().replace("\n", " ");
        let path_to_pdf = format!("./pdfs/{pdf_title}.pdf");
        let query_key = format!("{pdf_title}: {prompt}"); 
        let query = match self.check_cache(&query_key, QueryType::PdfCompletion) {
            // If found in cache, retrieve the query
            Some(query) => {
                let mut query = query.clone(); 
                query.from_cache = true;
                self.bill.cache_retrievals += 1; 
                self.update_bill(None); 
                println!("--[Cached Answer]--");
                query
            },
            // If absent, send to OpenAI
            None => {
                let from_cache = false;
                println!("--[Sending to GPT]--");
                // Load the pdf from the provided file path, or else return to the caller a NotFoundError 
                let pdf = lopdf::Document::load(path_to_pdf).map_err(|_| {return Status::NotFoundError})?;
                let mut doc = String::new();
                for page in 1..=pdf.get_pages().len() {
                    let content = pdf.extract_text(&[page as u32]).expect("parse");
                    doc.push_str(&content);
                }
                let req = ChatCompletionRequest {
                    model: model.to_string(),
                    messages: vec![
                        ChatCompletionMessage {
                            role: MessageRole::user,
                            content: Some(prompt.clone()),
                            name: None,
                            function_call: None,
                        },
                    ], functions: None, function_call: None, temperature: Some(self.temperature)
                };
                let start_time = std::time::Instant::now();
                let response = match self.send_completion_request(req).await {Ok(res) => res, Err(e) => return Err(Status::Error(e.to_string()))};
                let process_time = start_time.elapsed().as_millis() as u64;
                println!("--[Completion received]--");
                // Build Query from Response
                let query = Query { prompt: prompt.clone(), response: response.clone(), model, process_time, query_type: QueryType::PdfCompletion, cost: response.cost(&model), temperature: self.temperature, from_cache };
                // Add Query to Cache
                self.cache_query(&query_key, &query);
                // Add data to Bill
                self.update_bill(Some(&query));

                println!("--[Bill now shows: ${:.2}]--", self.bill.cost / 100.0);
                println!("--[Took: {}ms, Cost: {:.4} cents]--", process_time, (query.response.cost(&query.model)));
                query
            },
        };
        println!("--[Got from or created to cache under key: \"{pdf_title} - {prompt}\"]--");
        println!("--");

        Ok(query)
    }

    /// Get a completion that runs the provided battery, using the responses in the current state of the local cache (the cache file should be in sync therewith). The key in cache for this query will be "{title} - {battery stamp}" <br>Only uses responses in Queries whose query_type is `QueryType::PdfCompletion`, ingoring `PromptCompletions` and `MetaCompletions`. <br><br>Sends in the response content of each query concatenated together in the end of the Battery. <br><br>Choose a battery that is intended to run a meta completion, not send a document. I recommend labeling these batteries with a non-semantic prefix "Met", such that Battery::MetaAnalysis is explicitly a battery to be used on meta-analysis pdfs, while Battery::MetAnalysis would be a meta-battery intended to run on a concatenation of responses on many documents. <br><br>Always overwrites previous meta Queries.
    pub async fn meta_complete_cache(&mut self, title: String, battery_type: Battery, model: Option<GptModel>) -> Result<Query, Status> {
        
        println!("\n--🗳️  Meta Completion");
        
        let model = match model {Some(m) => m, None => self.model};
        let battery_label = battery_type.as_prompt_stamp();
        let query_key = format!("{title} - {battery_label}");
        
        let query = {
                let from_cache = false;
                // Convert the cache's PdfCompletions into a list of responses
                let mut build_input = String::new();
                let mut iter = 0;
                println!("--[Combining Essays:");
                for (_cache_key, query) in &self.cache {
                    if let QueryType::PdfCompletion = query.query_type {
                        iter += 1;
                        build_input.push_str(format!("\n\n{iter})\n").as_str());
                        let content = query.response.choices[0].clone().message.content.expect("presence of content field in GPT-response");
                        build_input.push_str(content.as_str());
                    }
                    //if iter == 3 {println!("--Current state of the input at 3:\n{build_input}");}
                }
                let input = build_input;
                println!("\n--Essays combined and ready for meta-completion.]--");

                println!("--[Sending to GPT]--");
                let req = ChatCompletionRequest {
                    model: model.to_string(),
                    messages: vec![
                        ChatCompletionMessage {
                            role: MessageRole::user,
                            content: Some(battery_type.to_prompt(input)),
                            name: None,
                            function_call: None,
                        },
                    ], functions: None, function_call: None, temperature: Some(self.temperature)
                };

                let start_time = std::time::Instant::now();
                let response = match self.send_completion_request(req).await {Ok(res) => res, Err(e) => return Err(Status::Error(e.to_string()))};
                let process_time = start_time.elapsed().as_millis() as u64;
                
                println!("--[Completion received]--");

                // Build Query from Response
                let query = Query { prompt: battery_type.as_prompt_stamp(), response: response.clone(), model, process_time, query_type: QueryType::MetaCompletion, cost: response.cost(&model), temperature: self.temperature, from_cache };
                self.cache_query(&query_key, &query); // Add Query to Cache
                self.update_bill(Some(&query)); // Add data to Bill
                println!("--[Bill now shows: ${:.2}]--", self.bill.cost / 100.0);
                println!("--[Took: {}ms, Cost: ¢{:.4}]--", process_time, (query.response.cost(&query.model)));
                query
        };

        println!("--[Created meta completion query to cache under key: \"{title} - {battery_label}\"]--");
        println!("--");
        Ok(query)

    }

    ///// Uses the provided model and battery, inserting into the battery a manually constructed input. This allows middle-processing, after Queries have been built up in cache, before sending their data for meta-analysis. <br>If you just want to run the battery on the current state of the cache, use `.meta_complete_cache()`
    //pub async fn meta_complete(&mut self, input: String, battery_type: Battery, model: Option<GptModel>) {}
}


use super::models::db::prelude::*;
use db::query_cache::*;
use sea_orm::ActiveValue;
use std::result::Result;
use std::error::Error as ErrorTrait;

/// Methods for coordinating the current cache state and the DB
impl OpenAIAccount {

    // Any time an OpenAIAccount is initiated, it is synced with the current state of the local cache files (assigned to constants)
    // The state of the cache in memory is updated, and saved to the cache file any time requests are made, due to the design of request methods.
    // At times, it may be necessary to engage in CR(U)D to the database, with regard to the current cache state (i.e. both in memory and in file) 
    // There are no Update methods because our data has no reason to be changed from its original state.

    /// Save current cache (the cache file & in-memory cache map which are synced) to the db, replacing those keys that already exist.
    pub async fn db_insert_cache(&self) -> std::result::Result<(), Box<dyn ErrorTrait>> {
        println!("🗄️  Saving cache to database...");
        let mut overwritten = false;
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL")?).await?;
        let mut models: Vec<ActiveModel> = vec![]; // initialize a vector
        models.reserve(self.cache.len()); // (optional) prepare memory ahead for length of the cache
        for (cache_key, query) in &self.cache {
            let query_key_hash = calculate_hash(cache_key);
            let extant_at_id = QueryCache::find().filter(Column::QueryKeyHash.eq(&query_key_hash)).one(&db).await.expect("Database check for query");
            if let Some(model) = extant_at_id {
                QueryCache::delete_by_id(model.rid).exec(&db).await.expect("success of deletion by id during db_insert_cache()");
                println!("🗄️  Model overwritten at query key hash: {query_key_hash}"); 
                overwritten = true;
                let graveyard = std::fs::OpenOptions::new().create(true).append(true).open("graveyard.json").expect("access to graveyard file");
                serde_json::to_writer_pretty(graveyard, &model).expect("Serialization of an overwritten model to the graveyard");
                
            }
            let model = ActiveModel { 
                timestamp: ActiveValue::Set(chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string()), 
                model: ActiveValue::Set(query.model.to_string()), 
                temperature: ActiveValue::Set(query.temperature), 
                prompt: ActiveValue::Set(query.prompt.to_string()),
                query_key: ActiveValue::Set(cache_key.to_string()), 
                prompt_tokens: ActiveValue::Set(query.response.usage.prompt_tokens), 
                completion_tokens: ActiveValue::Set(query.response.usage.completion_tokens), 
                total_tokens: ActiveValue::Set(query.response.usage.total_tokens), 
                process_time: ActiveValue::Set(query.process_time as i32), 
                response: ActiveValue::Set(serde_json::to_value(query.response.clone()).expect("conversion to JSON value of query.response")), 
                cost: ActiveValue::Set(query.cost),
                query_key_hash: ActiveValue::Set(query_key_hash), 
                rid: ActiveValue::NotSet
            };
            models.push(model)
        }
        let _res = QueryCache::insert_many(models).exec(&db).await?;
        if overwritten {println!("🪦  Any overwritten models can be recovered in graveyard file.")};
        println!("🗄️  Cache saved to database.");
        Ok(())
    }

    /// Insert the Query found at the provided cache_key from local cache into the database. <br> Returns the `rid` of the inserted query as Some if the provided cache_key has a corresponding value, or None if it does not.
    pub async fn db_insert_query(&self, cache_key: String) -> Option<i32> {
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL").expect("database env var")).await.expect("database connection");
        let query = match self.cache.get(&cache_key) {Some(s)=>s, None=> return None};
        
        let model = ActiveModel { 
            timestamp: ActiveValue::Set(chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string()), 
            model: ActiveValue::Set(query.model.to_string()), 
            temperature: ActiveValue::Set(query.temperature), 
            prompt: ActiveValue::Set(query.prompt.to_string()),
            query_key: ActiveValue::Set(cache_key.to_string()), 
            prompt_tokens: ActiveValue::Set(query.response.usage.prompt_tokens), 
            completion_tokens: ActiveValue::Set(query.response.usage.completion_tokens), 
            total_tokens: ActiveValue::Set(query.response.usage.total_tokens), 
            process_time: ActiveValue::Set(query.process_time as i32), 
            response: ActiveValue::Set(serde_json::to_value(query.response.clone()).expect("conversion to JSON value of query.response")), 
            cost: ActiveValue::Set(query.cost),
            query_key_hash: ActiveValue::Set(calculate_hash(&cache_key)), 
            rid: ActiveValue::NotSet
        };

        let res = QueryCache::insert(model).exec(&db).await.expect("insertion of ActiveModel to db during .db_insert_query()");

        let id = res.last_insert_id;
        println!("🗄️  Inserted into database query \"{key}\"", key = cache_key);
        Some(id)
    }


    /// Find all in db, convert models to queries, insert queries into the local cache according to query_key, overwriting if `overwrite` is `true` or skipping if not, then overwrite the cache file with the new state of the cache.  Returns the previous state of the cache, before db addition.
    pub async fn db_read_to_cache(&mut self, overwrite: bool) -> Result< HashMap<String,Query> , Box<dyn ErrorTrait> > {
        println!("🗄️  Reading database into cache...");
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL")?).await?;
        let previous_state = self.cache.clone();
        let models = QueryCache::find().all(&db).await?;

        // ? Here we are repeating the code for .cache_query(), with some adjustments, mainly so that we only overwrite and save to file once
        for model in models { 
            let query = model.to_query();
            // Make the key uniform if it is a prompt completion
            let query_key = if let QueryType::PromptCompletion = query.query_type {model.query_key.to_lowercase().replace("\n", " ")} else {model.query_key.to_string()};
            if !overwrite { match self.check_cache(&query_key, query.query_type) { Some(_) => continue, None => ()};  };
            // Add to self.cache
            self.cache.insert(query_key, query);
        }

        // Save the state of self.cache to file
        let cache = match fs::OpenOptions::new().create(true).write(true).open(CACHE_FILEPATH) {Ok(f)=>f, Err(e)=>panic!("Could not cache query at {CACHE_FILEPATH}, due to error:  ❌  {e}")};
        serde_json::to_writer_pretty(&cache, &self.cache).expect("Serialization of cache to cache file during db_read_to_cache");

        println!("🗄️  Database added to cache.");
        Ok(previous_state)
    }

    /// Returns Some(Model) if a row is found with the given key, else None.
    pub async fn db_read_one_by_cache_key(&self, cache_key: String) -> Option<Model> {
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL").expect("Database env var")).await.expect("Database connection");
        let model = QueryCache::find().filter(Column::QueryKey.eq(cache_key)).one(&db).await.expect("Database .find() call response success");
        model
    }

    pub async fn db_delete_one_by_id(&self, id: i32) -> Result<(), Box<dyn ErrorTrait>> {
        println!("🗄️  Deleting by id: {id}");
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL")?).await?;
        let _res = QueryCache::delete_by_id(id).exec(&db).await?;
        Ok(())
    }

    pub async fn db_delete_all(&self) -> Result<(), Box<dyn ErrorTrait>> {
        println!("🗄️  Delete database requested...");
        
        let mut line = String::new();
        println!("🗄️  Press Enter to continue...");
        let _input = std::io::stdin().read_line(&mut line).expect("Failed to read line");


        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL")?).await?;
        let _res = QueryCache::delete_many().exec(&db).await?;
        println!("🗄️  Database cleared.\n");
        Ok(())
    }

    pub async fn db_read_all(&self) -> Result< HashMap<String,Query> , Box<dyn ErrorTrait> > {
        println!("🗄️  Read all from database requested...");
        let db: DatabaseConnection = Database::connect(dotenvy::var("DATABASE_URL").expect("Database env var")).await.expect("Database connection");

        let mut db_cache: HashMap<String, Query> = HashMap::new();
        let models = QueryCache::find().all(&db).await?;
        for model in models {
            db_cache.extend([ ( model.query_key.clone(), model.to_query())])
        }

        Ok(db_cache)
    }

}


#[derive(Debug)]
pub enum Status {
    Success,
    Error(String),
    NotFoundError,
    OpenAIError,
    APIReachedLimit,

}

/// Machinery for the fundamental request-response process
impl OpenAIAccount {

    pub async fn send_completion_request(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, APIError> {
        let res = self.post("/chat/completions", &req).await?;
        let r = res.json::<ChatCompletionResponse>().await;
        match r { Ok(r) => Ok(r), Err(e) => Err(self.new_error(e)) }
    }

    fn new_error(&self, err: reqwest::Error) -> APIError {
        APIError { message: err.to_string() }
    }

    pub async fn post<T: serde::ser::Serialize>(&self, path: &str, params: &T) -> Result<Response, APIError> {
        let client = reqwest::Client::new();
        let url = format!("{API_URL_V1}{path}");
        let res = client
            .post(&url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, "Bearer ".to_owned() + &self.api_key)
            .json(&params)
            .send()
            .await;
        match res {
            Ok(res) => match res.status().is_success() { true => Ok(res), false => Err(APIError { message: format!(  "{}: {}", res.status(), res.text().await.unwrap()   ) })  }, 
            Err(e) => Err(self.new_error(e)),
        }
    }
}