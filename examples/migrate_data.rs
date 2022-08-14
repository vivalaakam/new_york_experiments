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

    let start = 1566259200;

    for i in 0..1000 {
        println!("{}, {}", start + i * 86400, i);
        let resp = get_candles("XRPUSDT".to_string(), 5, start + i * 86400, 36, None).await;

        for row in resp {
            let hash = hash_md5(format!("{:?}", row.history));
            let _ = query(
                r#"INSERT INTO candles_cache ( id, input_data, score, day, start_time, hash ) VALUES ( $1 , $2, $3, $4, $5, $6 )"#,
            )
                .bind(format!("{}", row.start_time))
                .bind(row.history)
                .bind(row.max_profit_12)
                .bind((start + i * 86400) as i64)
                .bind(row.start_time as i64)
                .bind(hash)
                .execute(&pool)
                .await;
        }
    }
}
