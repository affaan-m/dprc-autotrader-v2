use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteResponse {
    pub data: QuoteData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteData {
    pub in_amount: String,
    pub out_amount: String,
    pub price_impact: f64,
    pub minimum_out_amount: String,
}

#[derive(Debug, Deserialize)]
pub struct SwapResponse {
    pub data: SwapData,
}

#[derive(Debug, Deserialize)]
pub struct SwapData {
    pub transaction: String,
}

pub struct JupiterDex {
    client: Client,
    rpc_client: RpcClient,
    api_key: String,
    slippage: f64,
}

impl JupiterDex {
    pub fn new(rpc_url: &str, api_key: String, slippage: f64) -> Self {
        Self {
            client: Client::new(),
            rpc_client: RpcClient::new(rpc_url.to_string()),
            api_key,
            slippage,
        }
    }

    pub async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<QuoteResponse> {
        let url = format!(
            "https://price.jup.ag/v4/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            input_mint, output_mint, amount, (self.slippage * 100.0) as u32
        );

        let response = self.client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?
            .json::<QuoteResponse>()
            .await?;

        Ok(response)
    }

    pub async fn execute_swap(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        wallet: &Keypair,
    ) -> Result<String> {
        // Get quote first
        let quote = self.get_quote(input_mint, output_mint, amount).await?;

        // Get swap transaction
        let url = "https://quote-api.jup.ag/v4/swap";
        let swap_request = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": wallet.pubkey().to_string(),
            "wrapUnwrapSOL": true
        });

        let response = self.client
            .post(url)
            .header("X-API-KEY", &self.api_key)
            .json(&swap_request)
            .send()
            .await?
            .json::<SwapResponse>()
            .await?;

        // Decode and sign transaction
        let transaction_data = STANDARD.decode(response.data.transaction)?;
        let mut transaction: Transaction = bincode::deserialize(&transaction_data)?;
        
        transaction.sign(&[wallet], self.rpc_client.get_latest_blockhash()?);

        // Send transaction
        let signature = self.rpc_client.send_transaction(&transaction)?;
        
        Ok(signature.to_string())
    }

    pub async fn check_token_tradable(&self, token_address: &str) -> Result<bool> {
        // Try to get quotes in both directions (token -> SOL and SOL -> token)
        let sol_mint = "So11111111111111111111111111111111111111112";
        let amount = 1_000_000; // 1 SOL in lamports

        let to_token = self.get_quote(sol_mint, token_address, amount).await;
        let from_token = self.get_quote(token_address, sol_mint, amount).await;

        Ok(to_token.is_ok() && from_token.is_ok())
    }
} 