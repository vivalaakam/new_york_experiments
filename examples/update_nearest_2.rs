use dotenv::dotenv;
use experiments::mae;
use futures::TryStreamExt;
use sqlx::{query, Error, PgPool};
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&*database_url)
        .await
        .expect("postgresql fails");

    let mut history = vec![];
    let mut next = Some("1611986700".to_string());
    let mut success = HashMap::new();
    let mut failure = HashMap::new();

    let mut rows = sqlx::query_as::<_, (String, Vec<f64>, i64)>(
        r#"SELECT id, input_data, day FROM candles_cache WHERE score <= 2"#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for row in rows {
        failure.insert(row.0.clone(), row.clone());
    }

    let mut rows = sqlx::query_as::<_, (String, Vec<f64>, i64)>(
        r#"SELECT id, input_data, day FROM candles_cache WHERE score > 2 "#,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for row in rows {
        success.insert(row.0.clone(), row.clone());
    }

    while next.is_some() {
        let next_id = next.unwrap();

        let parent = success.get(&next_id).unwrap();

        let mut top = vec![];
        for data in failure.values() {
            let score = mae(&parent.1, &data.1);
            top.push((data.0.to_string(), score))
        }
        top.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let child = top.first().unwrap();

        let _ = query(r#"UPDATE candles_cache SET parent = $1 WHERE id = $2"#)
            .bind(parent.0.to_string())
            .bind(child.0.to_string())
            .execute(&pool)
            .await;

        history.push(parent.2);

        if history.len() > 10 {
            history = history[1..history.len()].to_vec();
        }
        let child = failure.get(&child.0).unwrap();
        let mut top = vec![];
        for data in success.values() {
            if !history.contains(&data.2) {
                let score = mae(&child.1, &data.1);
                top.push((data.0.to_string(), score))
            }
        }
        top.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let next_row = top.first().unwrap();

        let _ = query(r#"UPDATE candles_cache SET child = $1 WHERE id = $2"#)
            .bind(next_row.0.to_string())
            .bind(parent.0.to_string())
            .execute(&pool)
            .await;

        success.remove(&parent.0.to_string());
        failure.remove(&child.0.to_string());

        next = Some(next_row.0.to_string());

        println!("next: {:?} {:?}", next, history);
    }
}
