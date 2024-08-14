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
        let client = Client::new();
        // let blxr_pubkey = Pubkey::from_str(BLXR_DEST).map_err(|e| {
        //     ClientError::from(ClientErrorKind::Custom(format!(
        //         "Invalid BLXR pubkey: {}",
        //         e
        //     )))
        // })?;

        // let transfer_ix =
        //     system_instruction::transfer(&self.signer().pubkey(), &blxr_pubkey, TIP_AMOUNT);
        // let recent_blockhash = self.rpc_client.get_latest_blockhash().await.map_err(|e| {
        //     ClientError::from(ClientErrorKind::Custom(format!(
        //         "Failed to get recent blockhash: {}",
        //         e
        //     )))
        // })?;

        // let mut transfer_tx = Transaction::new_signed_with_payer(
        //     &[transfer_ix],
        //     Some(&self.signer().pubkey()),
        //     &[&self.signer()],
        //     recent_blockhash,
        // );

        // transfer_tx.sign(&[&self.signer()], recent_blockhash);

        let tx_data = base64::prelude::BASE64_STANDARD.encode(
            bincode::serialize(transaction).map_err(|e| {
                ClientError::from(ClientErrorKind::Custom(format!(
                    "Bincode serialization error: {}",
                    e
                )))
            })?,
        );

        // let transfer_tx_data = base64::prelude::BASE64_STANDARD.encode(
        //     bincode::serialize(&transfer_tx).map_err(|e| {
        //         ClientError::from(ClientErrorKind::Custom(format!(
        //             "Bincode serialization error for transfer tx: {}",
        //             e
        //         )))
        //     })?,
        // );

        let body = json!({
            "transactions": vec![tx_data, /* transfer_tx_data*/]
        });

        println!("auth token {}", auth_token);
        println!("body {}", body);

        let response: serde_json::Value = client
            .post(URL)
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
