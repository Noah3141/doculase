//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "query_cache")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub rid: i32,
    pub timestamp: String,
    pub model: String,
    #[sea_orm(column_type = "Float")]
    pub temperature: f32,
    pub prompt: String,
    #[sea_orm(column_type = "custom(\"LONGTEXT\")")]
    pub query_key: String,
    #[sea_orm(unique)]
    pub query_key_hash: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub process_time: i32,
    pub response: Json,
    #[sea_orm(column_type = "Float")]
    pub cost: f32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
