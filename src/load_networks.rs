use serde_json::json;

use crate::{NeatNetworks, Parse};

pub async fn load_networks(parse: &Parse, inputs: usize) -> Vec<NeatNetworks> {
    let mut networks = vec![];

    let mut cont = true;
    let mut skip = 0;

    while cont == true {
        let result = parse
            .query::<NeatNetworks, _, _>(
                "NeatNetworks",
                json!({"inputs": inputs, "outputs": 2}),
                Some(10),
                Some(skip),
                None,
            )
            .await;

        if result.results.len() < 10 {
            cont = false;
        }

        networks = [networks, result.results].concat();

        skip += 10
    }

    networks
}
