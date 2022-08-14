use dotenv::dotenv;
use experiments::mae;
use futures::TryStreamExt;
use sqlx::{query, Error, PgPool};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&*database_url)
        .await
        .expect("postgresql fails");

    let mut next = Some("1611986700".to_string());

    let mut history = vec![];

    while next.is_some() {
        let next_id = next.unwrap();
        let row = sqlx::query_as::<_, (String, Vec<f64>, i64)>(
            r#"SELECT id, input_data, day FROM candles_cache WHERE id = $1"#,
        )
        .bind(next_id.to_string())
        .fetch_one(&pool)
        .await;

        if row.is_err() {
            println!("{:?}", row);
            next = None;
            continue;
        }

        let master = row.unwrap();

        let mut rows = sqlx::query_as::<_, (String, Vec<f64>)>(
            r#"SELECT id, input_data FROM candles_cache WHERE score <= 2 AND parent IS NULL"#,
        )
        .bind(next_id.to_string())
        .fetch_all(&pool)
        .await
        .unwrap();

        let mut top = vec![];

        for row in rows {
            let score = mae(&master.1, &row.1);
            top.push((row.0, score))
        }

        top.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let _ = query(r#"UPDATE candles_cache SET parent = $1 WHERE id = $2"#)
            .bind(master.0.to_string())
            .bind(&top[0].0)
            .execute(&pool)
            .await;

        let _ = query(r#"UPDATE candles_cache SET checked = true WHERE id = $1"#)
            .bind(master.0.to_string())
            .execute(&pool)
            .await;

        history.push(master.2);

        if history.len() > 10 {
            history = history[1..history.len()].to_vec();
        }

        let mut rows = sqlx::query_as::<_, (String, Vec<f64>)>(
            r#"SELECT id, input_data FROM candles_cache WHERE score > 2 AND checked = false AND day != ANY($1)"#,
        )
            .bind(history.to_vec())
            .fetch_all(&pool).await.unwrap();

        let mut top = vec![];

        for row in rows {
            let score = mae(&master.1, &row.1);
            top.push((row.0, score))
        }

        top.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        next = match top.first() {
            None => None,
            Some(row) => Some(row.0.to_string()),
        };

        if let Some(id) = next.clone() {
            let _ = query(r#"UPDATE candles_cache SET child = $2 WHERE id = $1"#)
                .bind(master.0.to_string())
                .bind(id)
                .execute(&pool)
                .await;
        }

        println!("next: {:?} {:?}", next, history);
    }
}
