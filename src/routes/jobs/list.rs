use rocket::{serde::json::Json, State};
use sea_orm::DatabaseConnection;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {

}

#[post("/list", data = "<body>")]
pub async fn handler(db: &State<DatabaseConnection>, body: Json<Request>) -> Json<Response> {

    Json(todo!())
}