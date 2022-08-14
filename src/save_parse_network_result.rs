use log::debug;
use new_york_calculate_core::CalculateResult;
use serde_json::json;

use crate::{hash_md5, Parse};

pub async fn save_parse_network_result(
    parse: &Parse,
    network_id: String,
    applicant_id: String,
    result: CalculateResult,
) -> String {
    let result_id = hash_md5(format!("{}:{}", network_id.to_string(), applicant_id));
    let result = parse
        .create(
            "NeatNetworkResults",
            json!({
                "objectId": result_id,
                "networkId": network_id,
                "applicantId": applicant_id,
                "score": result.score,
                "wallet": result.wallet,
                "drawdown": result.drawdown,
                "balance": result.balance,
                "avgWait": result.avg_wait,
                "minBalance": result.min_balance,
                "baseReal": result.base_real,
                "baseExpected": result.base_expected,
                "successfulRatio": result.successful_ratio,
                "openedOrders": result.opened_orders,
                "executedOrders": result.executed_orders
            }),
        )
        .await;
    debug!("{}", result);
    result_id
}
