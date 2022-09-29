use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use new_york_calculate_core::{get_candles_with_cache, Indicators};
use new_york_calculate_core::indicators::IndicatorsInput;
use new_york_utils::make_id;
use serde_json::json;
use vivalaakam_neat_rs::{Genome, Organism};

use experiments::{
    create_applicant, get_keys_for_interval, get_now, get_score_fitness, load_networks,
    NeatNetworkApplicants, NeatNetworkApplicantType, Parse, save_parse_network_result,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of times to load candles
    #[clap(long, value_parser, default_value_t = 12)]
    lookback: u32,
    #[clap(long, value_parser, default_value_t = 1)]
    outputs: u32,
    #[clap(long, value_parser, default_value_t = 1)]
    inputs: u32,
    #[clap(long, value_parser, default_value_t = 0.5)]
    gain: f64,
    #[clap(long, value_parser, default_value_t = 8000.0)]
    balance: f64,
    #[clap(long, value_parser, default_value_t = 100.0)]
    stake: f64,
    #[clap(long, value_parser, default_value_t = 4)]
    lag: u32,
    #[clap(long, value_parser, default_value_t = 5)]
    interval: u32,
    #[clap(long, value_parser, default_value_t = 12)]
    days: u64,
    #[clap(long, value_parser)]
    from: Option<u64>,
    #[clap(long, value_parser)]
    ticker: String,
    #[clap(long, value_parser)]
    applicant_type: Option<String>,
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
    let inputs = args.inputs as usize;
    let outputs = args.outputs as usize;

    let applicant_type = match args.applicant_type {
        None => NeatNetworkApplicantType::Float,
        Some(v) => v.into(),
    };

    let networks = load_networks(&parse, inputs, outputs).await;

    let days = [6, 9, 12, 15, 18, 21, 24, 27, 30];

    for day in days {
        // let inputs = args.inputs as usize;
        let results = parse
            .query::<NeatNetworkApplicants, _, _>(
                "NeatNetworkApplicants",
                json!({"days": day, "inputs": inputs, "outputs": outputs, "ticker": args.ticker, "applicantType": applicant_type }),
                None,
                None,
                Some("-from".to_string()),
            )
            .await;

        let mut from = match results.results.first() {
            None => 1643846400,
            Some(applicant) => applicant.from + 86400,
        };

        let to = (next - day - 2) * 86400;

        while from <= to {
            let result = create_applicant(
                &parse,
                day as u64,
                Some(from),
                args.lag as usize,
                args.interval as usize,
                args.lookback as usize,
                args.gain,
                args.stake,
                inputs,
                outputs,
                Some(vec![1.005, 1.0075, 1.01, 1.0125, 1.025, 1.05]),
                Some(vec![50.0, 100.0, 150.0, 200.0]),
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
                applicant_type.clone(),
                args.ticker.to_string(),
                args.balance,
            )
                .await;

            println!("{}", result);

            let applicant = parse
                .get::<NeatNetworkApplicants, _, _>("NeatNetworkApplicants", result)
                .await;

            if applicant.is_none() {
                from += 86400;
                continue;
            }

            let applicant = applicant.unwrap();

            let keys = get_keys_for_interval(applicant.from, applicant.to);

            let mut candles = vec![];

            for key in keys {
                let new_candles = get_candles_with_cache(
                    applicant.ticker.to_string(),
                    applicant.interval,
                    key,
                    applicant.lookback,
                    Some(applicant.indicators.to_vec()),
                )
                    .await;
                candles = [candles, new_candles].concat();
            }

            candles.sort();

            let mut best = (Organism::new(Genome::default()), None);
            let epoch = candles.len();

            for network in &networks {
                let mut organism = Organism::new(network.network.to_string().into());
                organism.set_id(network.object_id.to_string());
                get_score_fitness(&mut organism, &candles, &applicant, epoch);

                if organism.get_fitness() > best.0.get_fitness() {
                    best = (organism, Some(network.object_id.to_string()))
                }
            }

            if let Some(ref object_id) = best.1 {
                let result = applicant.get_result(&best.0, &candles, epoch);

                let score = result.wallet * result.drawdown;

                let _result_id = save_parse_network_result(
                    &parse,
                    object_id.to_string(),
                    applicant.object_id.to_string(),
                    result,
                    true,
                    make_id(5),
                )
                    .await;

                println!("{} - {}", applicant.object_id.to_string(), score);
            }

            from += 86400;
        }
    }
}
