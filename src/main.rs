use dotenvy::dotenv;
use mongodb::{bson, bson::doc, sync::Client};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use std::env;

fn main() {
    dotenv().expect(".env file not found");
    let arkham_api_key = env::var("ARKHAM_API_KEY").unwrap();
    let address = "0x2eB5e5713A874786af6Da95f6E4DEaCEdb5dC246";

    let mut headers = HeaderMap::new();
    headers.insert("API-Key", HeaderValue::from_str(&arkham_api_key).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://api.arkhamintelligence.com/intelligence/address/{}/all",
        address
    );
    let res: Value = client
        .get(url)
        .headers(headers)
        .send()
        .unwrap()
        .json()
        .unwrap();

    let mongo_client = Client::with_uri_str("mongodb://localhost:27017").unwrap();
    let database = mongo_client.database("arkham_intelligence");
    let collection = database.collection("chain_data");
    for (chain, chain_data) in res.as_object().unwrap() {
        let chain_data_map = chain_data.as_object().unwrap();
        let arkham_entity = chain_data_map.get("arkhamEntity").unwrap();
        let arkham_label = chain_data_map.get("arkhamLabel").unwrap();

        let data = doc! {
            "address": chain_data_map["address"].as_str().unwrap(),
            "chain": chain,
            "arkhamEntity": bson::to_bson(arkham_entity).unwrap(),
            "arkhamLabel": bson::to_bson(arkham_label).unwrap(),
            "isUserAddress": chain_data_map["isUserAddress"].as_bool().unwrap(),
            "contract": chain_data_map["contract"].as_bool().unwrap(),
        };

        collection.insert_one(data, None).unwrap();
    }
}
