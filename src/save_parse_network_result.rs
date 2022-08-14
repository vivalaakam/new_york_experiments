use log::debug;
use new_york_calculate_core::CalculateResult;

use crate::{hash_md5, NeatNetworkResults, Parse};

pub async fn save_parse_network_result(
    parse: &Parse,
    network_id: String,
    applicant_id: String,
    result: CalculateResult,
) -> String {
    let result_id = hash_md5(format!("{}:{}", network_id.to_string(), applicant_id));

    let value = NeatNetworkResults {
        object_id: result_id.to_string(),
        network_id,
        applicant_id,
        score: result.score,
        wallet: result.wallet,
        drawdown: result.drawdown,
        balance: result.balance,
        avg_wait: result.avg_wait,
        min_balance: result.min_balance,
        base_real: result.base_real,
        base_expected: result.base_expected,
        successful_ratio: result.successful_ratio,
        opened_orders: result.opened_orders,
        executed_orders: result.executed_orders,
    };

    let result = parse.create("NeatNetworkResults", value).await;
    debug!("NeatNetworkResults: {result}");
    result_id
}
