#[macro_use] extern crate rocket;
pub mod utils;
pub mod routes;

use routes::{
    dev::{
        rust_openai_test
    },
};


#[launch]
async fn rocket() -> _ {
    
    utils::validation::validate_env_vars();

    
    rocket::build()
        .mount("/", routes![])
        .mount("/dev", routes![rust_openai_test::test])
}
