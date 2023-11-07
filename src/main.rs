#[macro_use] extern crate rocket;
pub mod utils;
pub mod routes;
pub mod env;

use env::*;

use routes::{
    dev::{
        rust_openai_test
    },
    documents,
    libraries,
    jobs
};
use sea_orm::Database;


#[launch]
async fn rocket() -> _ {
    
    // let config = rocket::Config {
    //     ..rocket::Config::debug_default()
    // };

    let env = env::validate_env_vars();

    let db = match Database::connect(env.get("DATABASE_URL").unwrap()).await {
    Ok(db) => db,
    Err(e) => panic!("Error connecting to DB: {e}"),
    };

    rocket::build()
        .manage(db)
        .mount("/dev", routes![rust_openai_test::test])
        .mount("/documents", routes![
                documents::add::handler,
                documents::list::handler,
                documents::remove::handler,
            ])
        .mount("/jobs", routes![
                jobs::add::handler,
                jobs::list::handler,
                jobs::cancel::handler,
            ])
        .mount("/libraries", routes![
                libraries::create::handler 
            ])
}
