use std::collections::HashMap;

use reqwest::{Client, Response};
use serde::{de, Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeatNetworks {
    pub object_id: String,
    pub network: String,
    pub inputs: usize,
    pub outputs: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeatNetworkResults {
    pub object_id: String,
    pub network_id: String,
    pub applicant_id: String,
    pub score: f64,
    pub wallet: f64,
    pub drawdown: f64,
    pub balance: f64,
    pub avg_wait: f64,
    pub min_balance: f64,
    pub base_real: f64,
    pub base_expected: f64,
    pub successful_ratio: f64,
    pub opened_orders: usize,
    pub executed_orders: usize,
    pub gain_ratio: HashMap<String, f64>,
    pub profit_ratio: HashMap<String, f64>,
    pub is_unique: bool,
    pub stream: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    object_id: Option<String>,
}

pub struct Parse {
    remote_url: String,
    app_id: String,
    rest_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Results<T> {
    pub count: Option<u64>,
    pub results: Vec<T>,
    pub error: Option<String>,
}

impl<T> Results<T> {
    pub fn new(results: Vec<T>, error: Option<String>) -> Results<T> {
        Results {
            count: None,
            results,
            error,
        }
    }
}

impl<T: de::DeserializeOwned + Clone> Results<T> {
    pub fn get_first(&self) -> Option<T> {
        match self.results.first() {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn get_results(&self) -> Vec<T> {
        self.results.clone()
    }

    pub fn get_errors(&self) -> Option<String> {
        self.error.clone()
    }

    pub fn get_count(&self) -> Option<u64> {
        self.count
    }
}

impl Parse {
    pub fn new(remote_url: String, app_id: String, rest_key: String) -> Self {
        Parse {
            remote_url,
            app_id,
            rest_key,
        }
    }

    pub async fn get<T, T1, T2>(&self, class_name: T1, id: T2) -> Option<T>
    where
        T: de::DeserializeOwned,
        T1: Into<String>,
        T2: Into<String>,
    {
        let result: Result<T, _> = Client::new()
            .get(
                &format!(
                    "{}/classes/{}/{}",
                    self.remote_url,
                    class_name.into(),
                    id.into()
                )
                .to_string(),
            )
            .header("X-Parse-Application-Id", self.app_id.to_string())
            .header("X-Parse-REST-API-Key", self.rest_key.to_string())
            .send()
            .await
            .unwrap()
            .json()
            .await;

        match result {
            Ok(obj) => Some(obj),
            Err(_) => None,
        }
    }

    pub async fn create<T, T1>(&self, class_name: T, value: T1) -> String
    where
        T: Into<String>,
        T1: Serialize,
    {
        let result = Client::new()
            .post(&format!(
                "{}/classes/{}",
                self.remote_url,
                class_name.into()
            ))
            .header("X-Parse-Application-Id", self.app_id.to_string())
            .header("X-Parse-REST-API-Key", self.rest_key.to_string())
            .json(&value)
            .send()
            .await;

        match result {
            Ok(resp) => resp.text().await.unwrap(),
            Err(err) => {
                println!("{}", err);
                return "".to_string();
            }
        }
    }

    pub async fn update<T, T1, T2, T3>(&self, class_name: T1, id: T2, value: T3) -> Option<T>
    where
        T: de::DeserializeOwned,
        T1: Into<String>,
        T2: Into<String>,
        T3: Serialize,
    {
        let result: Result<T, _> = Client::new()
            .put(&format!(
                "{}/classes/{}/{}",
                self.remote_url,
                class_name.into(),
                id.into()
            ))
            .header("X-Parse-Application-Id", self.app_id.to_string())
            .header("X-Parse-REST-API-Key", self.rest_key.to_string())
            .json(&value)
            .send()
            .await
            .unwrap()
            .json()
            .await;

        match result {
            Ok(obj) => Some(obj),
            Err(_) => None,
        }
    }

    pub async fn delete<T1, T2>(&self, class_name: T1, id: T2) -> Result<Response, reqwest::Error>
    where
        T1: Into<String>,
        T2: Into<String>,
    {
        Client::new()
            .delete(&format!(
                "{}/classes/{}/{}",
                self.remote_url,
                class_name.into(),
                id.into()
            ))
            .header("X-Parse-Application-Id", self.app_id.to_string())
            .header("X-Parse-REST-API-Key", self.rest_key.to_string())
            .send()
            .await
    }

    pub async fn query<T, T1, T2>(
        &self,
        class_name: T1,
        query: T2,
        limit: Option<usize>,
        skip: Option<usize>,
        order: Option<String>,
    ) -> Results<T>
    where
        T: de::DeserializeOwned,
        T1: Into<String>,
        T2: Serialize,
    {
        let result: Result<Results<T>, _> = Client::new()
            .post(&format!(
                "{}/classes/{}",
                self.remote_url,
                class_name.into()
            ))
            .header("X-Parse-Application-Id", self.app_id.to_string())
            .header("X-Parse-REST-API-Key", self.rest_key.to_string())
            .json(&json!({ "_method":"GET", "limit": limit.unwrap_or(1000), "order": order.unwrap_or("createdAt".to_string()), "skip":skip.unwrap_or(0), "where": query }))
            .send()
            .await
            .unwrap()
            .json()
            .await;

        match result {
            Ok(res) => res,
            Err(error) => {
                println!("{:?}", error);
                Results::new(vec![], None)
            }
        }
    }
}
