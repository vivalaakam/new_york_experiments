use new_york_calculate_core::{CalculateResult, Candle, Indicators};
use serde::{Deserialize, Serialize};
use vivalaakam_neat_rs::Organism;

use crate::get_result_float::get_result_float;
use crate::get_result_matrix::get_result_matrix;
use crate::get_result_steps::get_result_steps;
use crate::get_result_steps_iterate::get_result_steps_iterate;
use crate::get_result_steps_iterate_back::get_result_steps_iterate_back;
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
    pub gain_matrix: Vec<f64>,
    pub applicant_type: NeatNetworkApplicantType,
    pub ticker: String,
    pub indicators: Vec<Indicators>,
    pub balance: f64,
}

impl NeatNetworkApplicants {
    pub fn get_result(&self, organism: &Organism, candles: &Vec<Candle>, epoch: usize) -> CalculateResult {
        match self.applicant_type {
            NeatNetworkApplicantType::Float => {
                get_result_float(&organism, &candles, self.balance, self.lag, self.stake)
            }
            NeatNetworkApplicantType::Matrix => {
                get_result_matrix(&organism, &candles,self.balance, self.stake, &self.profit_matrix)
            }
            NeatNetworkApplicantType::Steps => {
                get_result_steps(&organism, &candles,self.balance, &self.profit_matrix, &self.gain_matrix)
            }
            NeatNetworkApplicantType::StepsIterate => {
                get_result_steps_iterate(&organism, &candles,self.balance, &self.profit_matrix, &self.gain_matrix, epoch)
            }
            NeatNetworkApplicantType::StepsIterateBack => {
                get_result_steps_iterate_back(&organism, &candles,self.balance, &self.profit_matrix, &self.gain_matrix, epoch)
            }
            _ => CalculateResult::default(),
        }
    }
}
