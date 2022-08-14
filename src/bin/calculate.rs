use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use new_york_calculate_core::get_candles_with_cache;
use sqlx::PgPool;
use vivalaakam_neat_rs::{Activation, Config, Genome, Organism};

use experiments::{Buffer, find_appropriate, get_now, save_network};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value_t = 12)]
    lookback: u32,
    #[clap(long, value_parser, default_value_t = 50)]
    population: i32,
    #[clap(long, value_parser, default_value_t = 1.0)]
    gain: f64,
    #[clap(long, value_parser, default_value_t = false)]
    best: bool,
    #[clap(long, value_parser, default_value_t = 5)]
    interval: i32,
    #[clap(long, value_parser, default_value_t = 100)]
    stagnation: i32,
}

fn get_fitness(organism: &mut Organism, candles: &Vec<(Vec<f64>, i32)>) {
    let mut distance = candles.len() as i32;
    // let mut has_buy = 0;
    for candle in candles {
        let result = organism.activate(candle.0.to_vec());
        if result[1] > result[0] {
            // has_buy += 1;
            distance -= (candle.1 - 1).abs();
        } else {
            distance -= (candle.1 - 0).abs();
        }
    }

    // if has_buy > (candles.len() as i32 / 100) * 5 {
    organism.set_fitness((distance as f64 / candles.len() as f64) * 100f64);
    // } else {
    //     organism.set_fitness(0f64);
    // }
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
    println!("{:?}", args);

    let population_size = args.population as usize;
    let look_back = args.lookback as usize;
    let inputs = look_back * 15;
    let next = (get_now() / 86400000) as u64;

    let mut population = vec![];

    let mut candles: Vec<(Vec<f64>, i32)> = vec![];

    for i in 0..92 {
        let start_time = (next - (95 - i)) * 86400;
        let new_candles = get_candles_with_cache("XRPUSDT".to_string(), args.interval as usize, start_time, args.lookback as usize, None).await;

        for candle in new_candles {
            candles.push((
                candle.history,
                if candle.max_profit_12 > args.gain {
                    1
                } else {
                    0
                },
            ))
        }
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
        connection_max: 100000,
        node_max: 10000,
        node_enabled: 0.25,
    };

    let mut parent: Option<Organism> = None;
    let mut stack = vec![];
    if args.best == true {
        let networks = sqlx::query_as::<_, (String, String)>(
            r#"SELECT id, network FROM networks WHERE inputs = $1 AND outputs = $2"#,
        )
            .bind(inputs as i64)
            .bind(2)
            .fetch_all(&pool)
            .await
            .unwrap();

        for network in networks {
            let genome: Genome = network.1.into();
            let mut organism = Organism::new(genome);
            organism.set_id(network.0);
            get_fitness(&mut organism, &candles);

            stack.push(organism);
        }
    }

    stack.sort_by(|a, b| a.get_fitness().partial_cmp(&b.get_fitness()).unwrap());
    if stack.len() > population_size {
        stack = stack[stack.len() - population_size..stack.len()].to_vec();
    }

    if stack.len() > 0 {
        if let Some(last) = stack.last() {
            parent = Some(last.clone());
        }
    }

    while population.len() < population_size * 10 {
        let mut organism = match parent.as_ref() {
            None => Organism::new(Genome::generate_genome(
                inputs,
                2,
                vec![],
                Some(Activation::Relu),
                &config,
            )),
            Some(organism) => organism.mutate(None, &config).unwrap(),
        };

        get_fitness(&mut organism, &candles);
        population.push(organism);
    }

    population.sort();
    population = population[0..population_size].to_vec();

    let mut target = match population.first() {
        None => candles.len() as f64 * 0.25,
        Some(organism) => organism.get_fitness() * 1.001,
    };

    let mut epoch = 0;
    println!("target: {target}");
    let mut buffer = Buffer::new(10);
    loop {
        let start = get_now();
        let mut new_population = vec![];

        for i in 0..population.len() {
            let child = find_appropriate(&population, i);

            match population[i].mutate(child, &config) {
                None => {}
                Some(organism) => {
                    let mut organism = organism;
                    get_fitness(&mut organism, &candles);
                    new_population.push(organism)
                }
            }
        }

        population = [population, new_population].concat();
        population.sort();
        population = population[0..population_size].to_vec();

        let duration = (get_now() - start) as f64 / 1000.0;
        buffer.push(duration);

        if let Some(best) = population.first_mut() {
            println!(
                "{epoch: >6} ({: >6} - {:.8}): {:.8} {: >3} ( dur: {:.3}, avg: {:.3} )",
                candles.len(),
                target,
                best.get_fitness(),
                best.get_stagnation(),
                duration,
                buffer.avg()
            );

            if best.get_fitness() > target {
                let id = save_network(&pool, best, parent.as_ref(), inputs, 2, target, false, None)
                    .await;

                target = best.get_fitness() * 1.001;
                best.set_id(id.to_string());
                parent = Some(best.clone());
                stack.push(best.clone());
            }

            best.inc_stagnation();

            if best.get_stagnation() > args.stagnation as usize {
                if best.get_id().is_none() {
                    let _ =
                        save_network(&pool, best, parent.as_ref(), inputs, 2, target, true, None)
                            .await;
                }

                parent = stack.pop();
                population = vec![];
                while population.len() < population_size * 10 {
                    let mut organism = match parent.as_ref() {
                        Some(parent) => parent.clone().mutate(None, &config).unwrap(),
                        None => Organism::new(Genome::generate_genome(
                            inputs,
                            2,
                            vec![],
                            Some(Activation::Relu),
                            &config,
                        )),
                    };

                    get_fitness(&mut organism, &candles);
                    population.push(organism);
                }

                population.sort();
                population = population[0..population_size].to_vec();
            }
        }

        epoch += 1;
    }
}
