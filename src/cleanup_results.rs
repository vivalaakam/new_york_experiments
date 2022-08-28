use crate::{NeatNetworkResults, Parse};
use serde_json::json;

pub async fn cleanup_results(parse: &Parse, row: &NeatNetworkResults) {
    parse
        .delete("NeatNetworkResults", row.object_id.to_string())
        .await
        .expect("TODO: panic message");

    let networks_results = parse
        .query::<NeatNetworkResults, _, _>(
            "NeatNetworkResults",
            json!({
                "networkId": row.network_id
            }),
            Some(10000),
            None,
            None,
        )
        .await;

    if networks_results.results.len() == 0 {
        parse
            .delete("NeatNetworks", row.network_id.to_string())
            .await
            .expect("TODO: panic message");
    }
}
