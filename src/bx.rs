use base64::Engine;
use chrono::Local;
use reqwest::Client;
use serde_json::json;
use solana_client::client_error::{ClientError, ClientErrorKind, Result as ClientResult};
use solana_program::{pubkey::Pubkey, system_instruction};
use solana_sdk::{signature::Signature, signer::Signer, transaction::Transaction};
use std::{fs::OpenOptions, io::Write, str::FromStr};

use crate::Miner;

const TIP_AMOUNT: u64 = 1_000_00; // 0.0001 SOL in lamports
const BLXR_DEST: &str = "BLXRkyw6WFiuiQUFNk7HLPb8rDQKgDpTsgZ1V6Mxrmw1";
const URL: &str = "https://ore-ny.solana.dex.blxrbdn.com/api/v2/mine-ore";

impl Miner {
    pub async fn post_submit_v2(
        &self,
        transaction: &Transaction,
        _skip_pre_flight: bool,
        _use_staked_rpcs: bool,
        auth_token: &str,
    ) -> ClientResult<Signature> {
        println!("Starting post_submit_v2 function...");
        let client = Client::new();

        println!("Encoding transaction...");
        let tx_data = base64::prelude::BASE64_STANDARD.encode(
            bincode::serialize(transaction).map_err(|e| {
                println!("Error serializing transaction: {}", e);
                ClientError::from(ClientErrorKind::Custom(format!(
                    "Bincode serialization error: {}",
                    e
                )))
            })?,
        );

        let body = json!({
            "transactions": vec![tx_data]
        }); 

        println!("Auth token: {}", auth_token);
        println!("Request body: {}", body);

        println!("Sending POST request to {}...", URL);
        let response = client
            .post(URL)
            .json(&body)
            .header("Authorization", auth_token)
            .send()
            .await
            .map_err(|e| {
                println!("Request Error: {}", e);
                ClientError::from(ClientErrorKind::Custom(format!("Request error: {}", e)))
            })?;

        println!("Response status: {}", response.status());
        println!("Response headers: {:?}", response.headers());

        let response_text = response.text().await.map_err(|e| {
            println!("Error reading response body: {}", e);
            ClientError::from(ClientErrorKind::Custom(format!("Response body error: {}", e)))
        })?;

        println!("Response body: {}", response_text);

        let response_json: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
            println!("JSON parsing error: {}", e);
            ClientError::from(ClientErrorKind::Custom(format!("JSON parsing error: {}", e)))
        })?;

        println!("Parsed JSON response: {:?}", response_json);

        let signature = response_json["signature"].as_str().ok_or_else(|| {
            println!("Signature not found in response");
            ClientError::from(ClientErrorKind::Custom(
                "Signature not found in response".to_string(),
            ))
        })?;

        println!("Extracted signature: {}", signature);

        self.save_signature(signature)?;

        println!("Signature saved successfully");

        Signature::from_str(signature).map_err(|e| {
            println!("Error parsing signature: {}", e);
            ClientError::from(ClientErrorKind::Custom(format!(
                "Signature parsing error: {}",
                e
            )))
        })
    }

    fn save_signature(&self, signature: &str) -> ClientResult<()> {
        println!("Saving signature to file...");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("signatures.log")
            .map_err(|e| {
                println!("Error opening signatures.log: {}", e);
                ClientError::from(ClientErrorKind::Custom(format!("File open error: {}", e)))
            })?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "{}: {}", timestamp, signature).map_err(|e| {
            println!("Error writing to signatures.log: {}", e);
            ClientError::from(ClientErrorKind::Custom(format!("File write error: {}", e)))
        })?;

        println!("Signature saved successfully");
        Ok(())
    }
}