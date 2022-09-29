use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use new_york_calculate_core::indicators::IndicatorsInput;
use new_york_calculate_core::Indicators;

use experiments::{create_applicant, NeatNetworkApplicantType, Parse};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to load candles
    #[clap(long, value_parser, default_value_t = 12)]
    lookback: u32,
    #[clap(long, value_parser, default_value_t = 1.25)]
    gain: f64,
    #[clap(long, value_parser, default_value_t = 8000.0)]
    balance: f64,
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
    #[clap(long, value_parser)]
    ticker: String,
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
    let inputs = args.lookback as usize * 15;
    let outputs = 5;
    let result = create_applicant(
        &parse,
        args.days as u64,
        args.from,
        args.lag as usize,
        args.interval as usize,
        args.lookback as usize,
        args.gain,
        args.stake,
        inputs,
        outputs,
        None,
        Some(vec![25.0, 50.0, 75.0]),
        Some(vec![
            Indicators::Ema(IndicatorsInput::Close, 10),
            Indicators::Ema(IndicatorsInput::Close, 20),
            Indicators::Sma(IndicatorsInput::Close, 10),
            Indicators::Sma(IndicatorsInput::Close, 20),
            Indicators::Vwma(IndicatorsInput::Close, 20),
            Indicators::Hma(IndicatorsInput::Close, 9),
            Indicators::Macd(IndicatorsInput::Close, 12, 26, 9, 0),
            Indicators::Macd(IndicatorsInput::Close, 12, 26, 9, 1),
            Indicators::Macd(IndicatorsInput::Close, 12, 26, 9, 2),
            Indicators::Rsi(IndicatorsInput::Close, 14),
            Indicators::Rsi(IndicatorsInput::Close, 20),
            Indicators::Stoch(14, 3, 3, 0),
            Indicators::Stoch(14, 3, 3, 1),
            Indicators::Adx(14),
            Indicators::Rsi(IndicatorsInput::Close, 15),
            Indicators::BBands(IndicatorsInput::Close, 29, 2.0, 0),
            Indicators::BBands(IndicatorsInput::Close, 29, 2.0, 1),
            Indicators::BBands(IndicatorsInput::Close, 29, 2.0, 2),
            Indicators::Atr(14),
            Indicators::Cci(20),
            Indicators::Adx(30),
            Indicators::Ad,
            Indicators::Obv,
            Indicators::Vpt,
            Indicators::Cpr(0),
            Indicators::Cpr(1),
            Indicators::Cpr(2),
        ]),
        NeatNetworkApplicantType::Steps,
        args.ticker,
        args.balance
    )
    .await;

    println!("{}", result);
}
