use rocket::{serde::json::Json, State};
use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    user_id: String,
    library_id: String,
    file: File,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    contents: String,
    filename: String,
    size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZodDocument {
    authors: Vec<String>,
    doc_id: String,
    pub_date: String, // You can use a more specific type for pub_date if needed
    pub_source: String,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    added_file: String,
    document: ZodDocument,
    library_id: String,
    user_id: String,
    msg: Option<String>,
    num_doclets: i32,
    success: bool,
    file: File,
}


#[post("/add", data = "<body>")]
pub async fn handler(db: &State<DatabaseConnection>, body: Json<Request>) -> Json<Response> {

    Json(todo!())
}

