use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use new_york_calculate_core::get_candles_with_cache;
use serde_json::json;
use vivalaakam_neat_rs::{Genome, Organism};

use experiments::{
    create_applicant, get_keys_for_interval, get_now, get_result, get_score_fitness, load_networks,
    save_parse_network_result, NeatNetworkApplicants, Parse,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to load candles
    #[clap(long, value_parser, default_value_t = 12)]
    lookback: u32,
    #[clap(long, value_parser, default_value_t = 1.25)]
    gain: f64,
    #[clap(long, value_parser, default_value_t = 200.0)]
    stake: f64,
    #[clap(long, value_parser, default_value_t = 4)]
    lag: u32,
    #[clap(long, value_parser, default_value_t = 5)]
    interval: u32,
    #[clap(long, value_parser, default_value_t = 7)]
    days: u64,
    #[clap(long, value_parser)]
    from: Option<u64>,
}

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    dotenv().ok();

    let args = Args::parse();

    let parse = Parse::new(
        env::var("PARSE_REMOTE_URL").expect("PARSE_REMOTE_URL must be set"),
        env::var("PARSE_APP_ID").expect("PARSE_APP_ID must be set"),
        env::var("PARSE_REST_KEY").expect("PARSE_REST_KEY must be set"),
    );

    println!("{:?}", args);

    let next = (get_now() / 86400000) as u64;
    let inputs = args.lookback as usize * 15;
    let outputs = 5;
    let results = parse
        .query::<NeatNetworkApplicants, _, _>(
            "NeatNetworkApplicants",
            json!({"days": args.days}),
            None,
            None,
            Some("-from".to_string()),
        )
        .await;

    let networks = load_networks(&parse, inputs, outputs).await;

    let mut from = match results.results.first() {
        None => 1649030400,
        Some(applicant) => applicant.from + 86400,
    };

    let to = (next - args.days - 2) * 86400;

    while from <= to {
        let result = create_applicant(
            &parse,
            args.days as u64,
            Some(from),
            args.lag as usize,
            args.interval as usize,
            args.lookback as usize,
            args.gain,
            args.stake,
            inputs,
            outputs,
        )
        .await;

        println!("{}", result);

        let applicant = parse
            .get::<NeatNetworkApplicants, _, _>("NeatNetworkApplicants", result)
            .await;

        if applicant.is_none() {
            continue;
        }

        let applicant = applicant.unwrap();

        let keys = get_keys_for_interval(applicant.from, applicant.to);

        let mut candles = vec![];

        for key in keys {
            let new_candles = get_candles_with_cache(
                "XRPUSDT".to_string(),
                applicant.interval,
                key,
                applicant.lookback,
                None,
            )
            .await;
            candles = [candles, new_candles].concat();
        }

        candles.sort();

        let mut best = (Organism::new(Genome::default()), None);

        for network in &networks {
            let mut organism = Organism::new(network.network.to_string().into());
            organism.set_id(network.object_id.to_string());
            get_score_fitness(
                &mut organism,
                &candles,
                applicant.gain,
                applicant.lag,
                applicant.stake,
            );

            if organism.get_fitness() > best.0.get_fitness() {
                best = (organism, Some(network.object_id.to_string()))
            }
        }

        if let Some(ref object_id) = best.1 {
            let result = get_result(
                &best.0,
                &candles,
                applicant.gain,
                applicant.lag,
                applicant.stake,
            );

            let score = result.wallet * result.drawdown;

            let _result_id = save_parse_network_result(
                &parse,
                object_id.to_string(),
                applicant.object_id.to_string(),
                result,
            )
            .await;

            println!("{} - {}", applicant.object_id.to_string(), score);
        }

        from += 86400;
    }
}
