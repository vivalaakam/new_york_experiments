use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use vivalaakam_neat_rs::Config;

use experiments::{neat_score_applicant, on_add_network, NeatNetworkApplicants, Parse};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser, default_value_t = 50)]
    population: u32,
    #[clap(long, value_parser, default_value_t = false)]
    best: bool,
    #[clap(long, value_parser, default_value_t = false)]
    crossover: bool,
    #[clap(long, value_parser, default_value_t = 100)]
    stagnation: u32,
    #[clap(long, value_parser)]
    applicant: String,
}

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .is_test(true)
        .try_init();

    dotenv().ok();
    let args = Args::parse();

    println!("{:?}", args);

    let parse = Parse::new(
        env::var("PARSE_REMOTE_URL").expect("PARSE_REMOTE_URL must be set"),
        env::var("PARSE_APP_ID").expect("PARSE_APP_ID must be set"),
        env::var("PARSE_REST_KEY").expect("PARSE_REST_KEY must be set"),
    );

    let applicant = parse
        .get::<NeatNetworkApplicants, _, _>("NeatNetworkApplicants", args.applicant)
        .await;

    println!("{:?}", applicant);

    if applicant.is_none() {
        return;
    }

    let applicant = applicant.unwrap();

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

    let network_id = neat_score_applicant(
        &parse,
        applicant,
        config,
        args.best,
        args.crossover,
        args.stagnation as usize,
        args.population as usize,
    )
    .await;

    if let Some(network_id) = network_id {
        on_add_network(&parse, network_id).await;
    }
}
