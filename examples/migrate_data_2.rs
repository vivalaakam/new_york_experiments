use dotenv::dotenv;
use experiments::hash_md5;
use new_york_calculate_core::get_candles;
use sqlx::{query, PgPool};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&*database_url)
        .await
        .expect("postgresql fails");
}
