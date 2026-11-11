use anyhow::Result;
use dotenv::dotenv;
use rig::providers::openai;
use rig_solana_trader::SolanaTrader;
use solana_sdk::signature::{read_keypair_file, Keypair};
use std::env;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize logging with timestamps
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_ansi(true)
        .with_timestamp(true)
        .pretty()
        .init();

    info!("Starting Solana trading bot...");

    // Create OpenAI client and agent
    let openai_client = openai::Client::from_env();
    let agent = openai_client.agent("gpt-4").build();

    // Load wallet from private key
    let wallet = if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        let bytes = bs58::decode(&private_key)
            .into_vec()
            .expect("Invalid private key");
        Keypair::from_bytes(&bytes).expect("Invalid keypair")
    } else {
        // Fallback to local keypair file
        read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
            .expect("Failed to read keypair file")
    };

    info!("Wallet loaded: {}", wallet.pubkey());

    // Create trader instance
    let mut trader = SolanaTrader::new(
        agent,
        env::var("BIRDEYE_API_KEY")?,
        env::var("JUPITER_API_KEY")?,
        env::var("SOLANA_RPC_URL")?,
        wallet,
        env::var("MAX_POSITION_SIZE_SOL")?.parse()?,
        env::var("MIN_POSITION_SIZE_SOL")?.parse()?,
        "So11111111111111111111111111111111111111112".to_string(), // SOL mint
        env::var("JUPITER_SLIPPAGE")?.parse::<f64>()? as u64 * 100, // Convert percentage to bps
        env::var("TWITTER_USERNAME")?,
        env::var("TWITTER_COOKIES")?,
    );

    // Add allowed Twitter interactions
    info!("Configuring allowed Twitter interactions...");
    trader.add_allowed_twitter_interaction("vitalik".to_string());
    trader.add_allowed_twitter_interaction("solana".to_string());
    trader.add_allowed_twitter_interaction("aeyakovenko".to_string());
    trader.add_allowed_twitter_interaction("cryptogodfatha".to_string());
    trader.add_allowed_twitter_interaction("0xMert_".to_string());
    trader.add_allowed_twitter_interaction("DefiLlama".to_string());

    // Start trading loop
    info!("Starting trading loop...");
    info!("Press Ctrl+C to stop the bot");
    
    // Handle Ctrl+C gracefully
    let trader = Arc::new(trader);
    let trader_clone = Arc::clone(&trader);
    
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        result = trader_clone.start_trading_loop() => {
            if let Err(e) = result {
                tracing::error!("Trading loop error: {}", e);
            }
        }
    }

    info!("Bot stopped successfully");
    Ok(())
} 