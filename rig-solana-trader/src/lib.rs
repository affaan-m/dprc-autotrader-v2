pub mod dex;
pub mod market_data;
pub mod personality;
pub mod strategy;
pub mod twitter;
pub mod database;

use anyhow::Result;
use dex::jupiter::JupiterDex;
use market_data::birdeye::BirdEyeProvider;
use market_data::{AggregatedDataProvider, DataProvider};
use personality::StoicPersonality;
use rig::agent::Agent;
use rig::completion::CompletionModel;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::{Keypair, Signature}};
use strategy::TradingStrategy;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct SolanaTrader<M: CompletionModel> {
    strategy: TradingStrategy<M>,
    personality: StoicPersonality,
    data_provider: Arc<AggregatedDataProvider>,
    dex: JupiterDex,
    rpc: RpcClient,
    wallet: Keypair,
    base_token: String,
}

// ... rest of the implementation ... 