pub mod dex;
pub mod market_data;
pub mod personality;
pub mod strategy;
pub mod twitter;
pub mod database;

use dex::jupiter::JupiterDex;
use market_data::AggregatedDataProvider;
use personality::StoicPersonality;
use rig::completion::CompletionModel;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use strategy::TradingStrategy;
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