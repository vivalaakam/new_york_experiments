use std::env;
use std::fs::File;
use std::io::Read;

use dotenv::dotenv;
use log::LevelFilter;
use vivalaakam_neat_rs::{Connection, Genome, NeuronType, Node, Organism};

use experiments::{load_networks, on_add_network, save_parse_network, Parse};

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .is_test(true)
        .try_init();

    dotenv().ok();

    let parse = Parse::new(
        env::var("PARSE_REMOTE_URL").expect("PARSE_REMOTE_URL must be set"),
        env::var("PARSE_APP_ID").expect("PARSE_APP_ID must be set"),
        env::var("PARSE_REST_KEY").expect("PARSE_REST_KEY must be set"),
    );

    let mut file = File::open("tmp/networks.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json = serde_json::from_str::<Vec<Genome>>(&data).unwrap();

    for row in json {
        let organism = Organism::new(row);

        let network_id = save_parse_network(&parse, &organism, 63, 1).await;
        println!("{network_id}");
    }
}
