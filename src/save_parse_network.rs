use log::debug;
use vivalaakam_neat_rs::Organism;

use crate::{hash_md5, NeatNetworks, Parse};

pub async fn save_parse_network(
    parse: &Parse,
    best: &Organism,
    inputs: usize,
    outputs: usize,
) -> String {
    let network = best.as_json();
    let network_id = hash_md5(format!("{:?}", network));

    let value = NeatNetworks {
        object_id: network_id.to_string(),
        network,
        inputs,
        outputs,
    };

    let result = parse.create("NeatNetworks", value).await;

    debug!("NeatNetworks: {result}");

    network_id
}
