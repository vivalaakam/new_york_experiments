use dotenv::dotenv;
use log::LevelFilter;
use sqlx::{query, PgPool};
use std::env;
use std::sync::Mutex;

use experiments::{exists_file, get_now, hash_md5, read_from_file, write_to_file, Buffer};
use vivalaakam_neat_rs::{Activation, Config, Genome, Organism};

use clap::Parser;
use new_york_calculate_core::{
    get_candles, CalculateCommand, CalculateIter, CalculateResult, Candle,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to load candles
    #[clap(long, value_parser, default_value_t = 36)]
    lookback: u8,
    #[clap(long, value_parser, default_value_t = 1.0)]
    gain: f64,
    #[clap(long, value_parser, default_value_t = 25.0)]
    stake: f64,
    #[clap(long, value_parser, default_value_t = 1)]
    lag: u8,
}

fn get_fitness(organism: &mut Organism, candles: &Vec<Candle>, gain: f64, lag: usize, stake: f64) {
    let org = organism.clone();
    let buffer = Mutex::new(Buffer::new(lag));
    let mut calculate_iter = CalculateIter::new(
        &candles,
        3000.0,
        stake,
        gain / 100f64 + 1f64,
        0.5,
        5,
        1f64,
        0.0001f64,
        Box::new(move |candle, _ind| {
            let result = org.activate(candle.history.to_vec());
            let mut buffer = buffer.lock().unwrap();
            if result[1] > result[0] {
                buffer.push(1.0);
                if buffer.avg() > 0.5 {
                    CalculateCommand::BuyProfit
                } else {
                    CalculateCommand::None
                }
            } else {
                buffer.push(0.0);
                CalculateCommand::None
            }
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    let result: CalculateResult = calculate_iter.into();

    organism.set_fitness(result.wallet);
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
    let lag = args.lag as usize;
    let look_back = args.lookback as usize;
    let inputs = look_back * 15;
    let population_size = 50;
    let mut population = vec![];

    let mut candles: Vec<Candle> = vec![];
    let next = (get_now() / 86400000) as u64;
    let mut target = 20.0;
    let gain = args.gain;
    let stake = args.stake;

    for i in 0..90 {
        let start_time = (next - (94 - i)) * 86400;
        let filename = format!("tmp/{start_time}.cbor");
        if exists_file(filename.as_str()) == false {
            let new_candles =
                get_candles("XRPUSDT".to_string(), 5, start_time, look_back, None).await;
            write_to_file(filename.as_str(), new_candles);
        }

        let new_candles = read_from_file(filename.as_str());

        candles = [candles, new_candles].concat();
    }

    candles.sort();

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

    while population.len() < population_size {
        let genome = Genome::generate_genome(inputs, 2, vec![], Some(Activation::Relu), &config);
        let mut organism = Organism::new(genome);
        get_fitness(&mut organism, &candles, gain, lag, stake);
        population.push(organism);
    }

    population.sort();

    let mut epoch = 0;
    println!("target: {target}, next:{}", next);
    let mut parent = None;
    let mut buffer = Buffer::new(10);
    loop {
        let start = get_now();
        let mut new_population = vec![];

        for i in 0..population.len() {
            let j = if i > 0 {
                (i - 1) / 2
            } else {
                population.len() - 1
            };

            match population[i].mutate(&population[j], &config) {
                None => {}
                Some(organism) => new_population.push(organism),
            }
        }

        for organism in new_population.iter_mut() {
            get_fitness(organism, &candles, gain, lag, stake);
        }

        population = [population, new_population].concat();
        population.sort();
        population = population[0..population_size].to_vec();

        let duration = (get_now() - start) as f64 / 1000.0;
        buffer.push(duration);
        println!(
            "{epoch} ({} - {:.8}): {:.8} ( dur: {:.3}, avg: {:.3} )",
            candles.len(),
            target,
            population[0].get_fitness(),
            duration,
            buffer.avg()
        );
        epoch += 1;

        if population[0].get_fitness() > target {
            let network = population[0].as_json();
            let id = hash_md5(format!("{:?}", network));
            let _ = query(
                r#"INSERT INTO networks ( id, inputs, outputs, network, last_id, target, score, parent, type ) VALUES ( $1 , $2, $3, $4, $5, $6, $7, $8, 'score' )"#,
            )
                .bind(id.to_string())
                .bind(inputs as i32)
                .bind(2)
                .bind(network)
                .bind(next.to_string())
                .bind(target)
                .bind(population[0].get_fitness())
                .bind(parent)
                .execute(&pool)
                .await;

            parent = Some(id);

            target = population[0].get_fitness() * 1.05;
        }
    }
}
