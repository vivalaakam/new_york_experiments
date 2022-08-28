use new_york_calculate_core::{CalculateResult, Candle};
use serde::{Deserialize, Serialize};
use vivalaakam_neat_rs::Organism;

use crate::get_result_float::get_result_float;
use crate::get_result_matrix::get_result_matrix;
use crate::neat_network_applicant_type::NeatNetworkApplicantType;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeatNetworkApplicants {
    pub object_id: String,
    pub from: u64,
    pub to: u64,
    pub days: u64,
    pub high_score: f64,
    pub lookback: usize,
    pub gain: f64,
    pub stake: f64,
    pub lag: usize,
    pub interval: usize,
    pub touches: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub profit_matrix: Vec<f64>,
    pub applicant_type: NeatNetworkApplicantType,
    pub ticker: String,
}

impl NeatNetworkApplicants {
    pub fn get_result(&self, organism: &Organism, candles: &Vec<Candle>) -> CalculateResult {
        match self.applicant_type {
            NeatNetworkApplicantType::Float => {
                get_result_float(&organism, &candles, self.lag, self.stake)
            }
            NeatNetworkApplicantType::Matrix => {
                get_result_matrix(&organism, &candles, self.stake, &self.profit_matrix)
            }
            _ => CalculateResult::default(),
        }
    }
}
