use log::debug;
use serde_json::json;
use vivalaakam_neat_rs::Organism;

use crate::{hash_md5, Parse};

pub async fn save_parse_network(
    parse: &Parse,
    best: &Organism,
    inputs: usize,
    outputs: usize,
) -> String {
    let network = best.as_json();
    let network_id = hash_md5(format!("{:?}", network));
    let result = parse
        .create(
            "NeatNetworks",
            json!({"objectId": network_id.to_string(), "network": network, "inputs": inputs, "outputs": outputs}),
        )
        .await;

    debug!("{}", result);

    network_id
}
