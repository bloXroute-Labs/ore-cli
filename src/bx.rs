use base64::Engine;
use chrono::Local;
use reqwest::Client;
use serde_json::json;
use solana_client::client_error::{ClientError, ClientErrorKind, Result as ClientResult};
use solana_sdk::{signature::Signature, transaction::Transaction};
use std::{fs::OpenOptions, io::Write, str::FromStr};

use crate::Miner;

impl Miner {
    pub async fn post_submit_v2(
        &self,
        transaction: &Transaction,
        skip_pre_flight: bool,
        use_staked_rpcs: bool,
        auth_token: &str,
    ) -> ClientResult<Signature> {
        let client = Client::new();
        let url = "ny.solana.dex.blxrbdn.com";

        let tx_data = base64::prelude::BASE64_STANDARD.encode(
            bincode::serialize(transaction).map_err(|e| {
                ClientError::from(ClientErrorKind::Custom(format!(
                    "Bincode serialization error: {}",
                    e
                )))
            })?,
        );

        let body = json!({
            "transaction": {
                "content": tx_data
            },
            "skipPreFlight": skip_pre_flight,
            "useStakedRPCs": use_staked_rpcs,
            "tip": 5000
        });

        println!("auth token {}", auth_token);

        let response: serde_json::Value = client
            .post(url)
            .json(&body)
            .header("Authorization", auth_token)
            .send()
            .await
            .map_err(|e| {
                println!("Request Error: {}", e);
                ClientError::from(ClientErrorKind::Custom(format!("Request error: {}", e)))
            })?
            .json()
            .await
            .map_err(|e| {
                println!("Request JSON error: {}", e);
                ClientError::from(ClientErrorKind::Custom(format!(
                    "JSON deserialization error: {}",
                    e
                )))
            })?;

        println!("response {}", response);

        let signature = response["signature"].as_str().ok_or_else(|| {
            ClientError::from(ClientErrorKind::Custom(
                "Signature not found in response".to_string(),
            ))
        })?;

        self.save_signature(signature)?;

        Signature::from_str(signature).map_err(|e| {
            ClientError::from(ClientErrorKind::Custom(format!(
                "Signature parsing error: {}",
                e
            )))
        })
    }
    fn save_signature(&self, signature: &str) -> ClientResult<()> {
        // Method 1: Using standard Rust file I/O
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("signatures.log")
            .map_err(|e| {
                ClientError::from(ClientErrorKind::Custom(format!("File open error: {}", e)))
            })?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "{}: {}", timestamp, signature).map_err(|e| {
            ClientError::from(ClientErrorKind::Custom(format!("File write error: {}", e)))
        })?;

        Ok(())
    }
}
