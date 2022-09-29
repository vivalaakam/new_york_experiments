use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use new_york_calculate_core::{get_candles_with_cache, Indicators};
use serde_json::json;

use experiments::{create_applicant, get_keys_for_interval, get_now, get_score_fitness, load_networks, save_parse_network_result, NeatNetworkApplicantType, NeatNetworkApplicants, Parse, NeatNetworkResults};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value_t = 1)]
    outputs: u32,
    #[clap(long, value_parser, default_value_t = 63)]
    inputs: u32,
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

    let inputs = args.inputs as usize;
    let outputs = args.outputs as usize;


    let networks = load_networks(&parse, inputs, outputs).await;

    for network in networks {
        let networks_results = parse
            .query::<NeatNetworkResults, _, _>(
                "NeatNetworkResults",
                json!({
                "networkId": network.object_id.to_string()
            }),
                Some(10000),
                None,
                None,
            )
            .await;

        println!("{} - {}", network.object_id, networks_results.results.len());

        if networks_results.results.len() == 0 {
            parse
                .delete("NeatNetworks", network.object_id.to_string())
                .await
                .expect("TODO: panic message");
        }
    }
}
