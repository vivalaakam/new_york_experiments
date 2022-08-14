use std::env;

use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;

use experiments::{on_add_network, Parse};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser)]
    applicant: Option<String>,
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

    if args.applicant.is_none() {
        return;
    }

    on_add_network(&parse, args.applicant.unwrap()).await;
}
