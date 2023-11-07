use rocket::serde::json::Json;
use rust_openai::Query;


#[get("/rust_openai")]
pub async fn test() -> Json<Query> {

    let mut openai = rust_openai::OpenAIAccount::new(rust_openai::GptModel::Gpt35Turbo, 0.5);
    let res = openai
        .get_completion("Spell alphabet".to_string(), None)
        .await
        .expect("Failed to get completion");


    openai.db_insert_cache().await.expect("insertion");


    Json(res)

}

