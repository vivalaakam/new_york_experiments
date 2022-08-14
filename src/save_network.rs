use crate::hash_md5;
use sqlx::{query, Pool, Postgres};
use vivalaakam_neat_rs::Organism;

pub async fn save_network(
    pool: &Pool<Postgres>,
    best: &Organism,
    parent: Option<&Organism>,
    inputs: usize,
    outputs: usize,
    target: f64,
    stuck: bool,
    parents: Option<Vec<String>>,
) -> String {
    let network = best.as_json();
    let id = hash_md5(format!("{:?}", network));
    let parent_id = match parent.as_ref() {
        None => None,
        Some(p) => p.get_id(),
    };

    let _ = query(
        r#"INSERT INTO networks ( id, inputs, outputs, network, target, score, parent, stuck, parents, type ) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, 'score' )"#,
    )
        .bind(id.to_string())
        .bind(inputs as i32)
        .bind(outputs as i32)
        .bind(network)
        .bind(target)
        .bind(best.get_fitness())
        .bind(parent_id)
        .bind(stuck)
        .bind(parents)
        .execute(pool)
        .await;

    id
}
