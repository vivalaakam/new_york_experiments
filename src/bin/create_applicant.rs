use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;

use experiments::{create_applicant, Parse};

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
    #[clap(long, value_parser, default_value_t = 6)]
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

    let result = create_applicant(
        &parse,
        args.days as u64,
        args.from,
        args.lag as usize,
        args.interval as usize,
        args.lookback as usize,
        args.gain,
        args.stake,
    )
    .await;

    println!("{}", result);
}
