use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NeatNetworkApplicantType {
    Unknown,
    Float,
    Matrix,
}

impl Default for NeatNetworkApplicantType {
    fn default() -> Self {
        NeatNetworkApplicantType::Unknown
    }
}

impl From<String> for NeatNetworkApplicantType {
    fn from(value: String) -> Self {
        match value.as_str().to_lowercase().as_str() {
            "float" => NeatNetworkApplicantType::Float,
            "matrix" => NeatNetworkApplicantType::Matrix,
            _ => NeatNetworkApplicantType::Unknown,
        }
    }
}
