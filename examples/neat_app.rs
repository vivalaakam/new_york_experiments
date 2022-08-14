use dotenv::dotenv;
use log::LevelFilter;
use sqlx::{query, PgPool, Pool, Postgres};
use std::env;

use experiments::{get_now, hash_md5, mae, Buffer};
use vivalaakam_neat_rs::{Activation, Config, Genome, Organism};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to load candles
    #[clap(short, long, value_parser, default_value_t = 5)]
    count: u8,
}

struct App<'a> {
    pool: &'a Pool<Postgres>,
    candles: Vec<(Vec<f64>, Vec<f64>)>,
    next: Option<String>,
    parent: Option<String>,
    target_delta: f64,
    pub target: f64,
    pub population: Vec<Organism>,
    pub population_size: usize,
}

impl<'a> App<'a> {
    pub fn new(pool: &'a Pool<Postgres>, next: Option<String>) -> Self {
        App {
            pool,
            next,
            target_delta: 0.5,
            target: 0.0,
            population_size: 300,
            parent: None,
            candles: vec![],
            population: vec![],
        }
    }

    pub async fn load_more(&mut self) {
        let row = sqlx::query_as::<_, (Vec<f64>, f64, Vec<f64>, f64, String)>(
            r#"SELECT c1.input_data, c1.score, c2.input_data, c2.score, c1.child FROM candles_cache AS c1 LEFT JOIN candles_cache AS c2 ON c1.id = c2.parent WHERE c1.id = $1"#,
        )
            .bind(self.next.clone().unwrap())
            .fetch_one(self.pool)
            .await.unwrap();

        self.candles.push((
            row.0.to_vec(),
            if row.1 > 2.0 {
                vec![0.0, 1.0]
            } else {
                vec![1.0, 0.0]
            },
        ));

        self.candles.push((
            row.2.to_vec(),
            if row.3 > 2.0 {
                vec![0.0, 1.0]
            } else {
                vec![1.0, 0.0]
            },
        ));

        self.next = Some(row.4);
        self.target_delta *= 1.0001;
        self.target = self.candles.len() as f64 * self.target_delta;
    }

    pub fn best(&self) -> &Organism {
        self.population.first().unwrap()
    }

    pub fn get_next_id(&self) -> String {
        self.next.clone().unwrap_or_default().to_string()
    }
}

pub fn get_fitness(organism: &mut Organism, candles: &Vec<(Vec<f64>, Vec<f64>)>) {
    let mut distance: f64 = 0f64;

    for candle in candles {
        let output = organism.activate(candle.0.to_vec());
        distance += mae(&candle.1, &output)
    }

    organism.set_fitness(candles.len() as f64 / (1f64 + distance));
}

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&*database_url)
        .await
        .expect("postgresql fails");

    let args = Args::parse();

    let mut app = App::new(&pool, Some("1611986700".to_string()));

    for _ in 0..args.count {
        app.load_more().await;
    }

    let config = Config {
        add_node: 0.35,
        add_connection: 0.35,
        connection_enabled: 0.1,
        crossover: 0.1,
        connection_weight: 1.0,
        connection_weight_prob: 0.8,
        connection_weight_delta: 0.1,
        node_bias_prob: 0.25,
        node_activation_prob: 0.25,
        node_bias_delta: 0.1,
        node_bias: 1.0,
    };

    while app.population.len() < app.population_size {
        let genome = Genome::generate_genome(540, 2, vec![], Some(Activation::Sigmoid), &config);
        let mut organism = Organism::new(genome);
        get_fitness(&mut organism, &app.candles);
        app.population.push(organism);
    }

    app.population.sort();

    let mut epoch = 0;
    println!("target: {}, next:{}", app.target, app.get_next_id());
    let mut buffer = Buffer::new(10);
    loop {
        let start = get_now();
        let mut new_population = vec![];

        for i in 0..app.population.len() {
            let j = if i > 0 {
                (i - 1) / 2
            } else {
                app.population.len() - 1
            };

            match app.population[i].mutate(&app.population[j], &config) {
                None => {}
                Some(organism) => new_population.push(organism),
            }
        }

        for organism in new_population.iter_mut() {
            get_fitness(organism, &app.candles);
        }

        app.population = [app.population, new_population].concat();
        app.population.sort();
        app.population = app.population[0..app.population_size].to_vec();

        let duration = (get_now() - start) as f64 / 1000.0;
        buffer.push(duration);
        println!(
            "{epoch} ({} - {:.8}): {:.8} ( dur: {:.3}, avg: {:.3} )",
            app.candles.len(),
            app.target,
            app.best().fitness,
            duration,
            buffer.avg()
        );
        epoch += 1;

        if app.best().get_fitness() > app.target {
            let best = app.best();
            let network = best.as_json();
            let id = hash_md5(format!("{:?}", network));
            let _ = query(
                r#"INSERT INTO networks ( id, inputs, outputs, network, last_id, target, score, parent ) VALUES ( $1 , $2, $3, $4, $5, $6, $7, $8 )"#,
            )
                .bind(id.to_string())
                .bind(540)
                .bind(2)
                .bind(network)
                .bind(app.get_next_id())
                .bind(app.target)
                .bind(best.get_fitness())
                .bind(app.parent)
                .execute(&pool)
                .await;

            app.parent = Some(id);

            app.load_more().await;

            for organism in app.population.iter_mut() {
                get_fitness(organism, &app.candles);
            }
        }
    }
}
