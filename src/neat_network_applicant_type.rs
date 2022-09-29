use std::fmt::Formatter;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NeatNetworkApplicantType {
    Unknown,
    Float,
    Matrix,
    Steps,
    StepsIterate,
    StepsIterateBack,
}

impl Default for NeatNetworkApplicantType {
    fn default() -> Self {
        NeatNetworkApplicantType::Unknown
    }
}

impl std::fmt::Display for NeatNetworkApplicantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            NeatNetworkApplicantType::Float => "float".to_string(),
            NeatNetworkApplicantType::Matrix => "matrix".to_string(),
            NeatNetworkApplicantType::Steps => "steps".to_string(),
            NeatNetworkApplicantType::StepsIterate => "stepsIterate".to_string(),
            NeatNetworkApplicantType::StepsIterateBack => "stepsIterateBack".to_string(),
            NeatNetworkApplicantType::Unknown => "unknown".to_string()
        };

        write!(f, "{}", res)
    }
}

impl From<String> for NeatNetworkApplicantType {
    fn from(value: String) -> Self {
        match value.as_str().to_lowercase().as_str() {
            "float" => NeatNetworkApplicantType::Float,
            "matrix" => NeatNetworkApplicantType::Matrix,
            "steps" => NeatNetworkApplicantType::Steps,
            "stepsiterate" => NeatNetworkApplicantType::StepsIterate,
            "stepsiterateback" => NeatNetworkApplicantType::StepsIterateBack,
            _ => NeatNetworkApplicantType::Unknown,
        }
    }
}
