use anyhow::Result;
use rig_solana_trader::{
    database::DatabaseClient,
    market_data::{MarketDataProvider, vector_store::TokenAnalysis},
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize database
    let mongodb_uri = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_client = DatabaseClient::new(&mongodb_uri, "solana_trader").await?;

    // Initialize market data provider
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let mut market_data = MarketDataProvider::new(&openai_api_key, db_client).await?;

    // Test token analysis
    let test_tokens = vec![
        "So11111111111111111111111111111111111111112", // Wrapped SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
    ];

    for token in test_tokens {
        info!("Analyzing token {}", token);
        market_data.analyze_token(token).await?;
    }

    // Test similarity search
    let similar_tokens = market_data.find_similar_tokens("meme tokens with high social engagement", 2).await?;
    info!("Similar tokens: {:?}", similar_tokens);

    // Test sentiment analysis
    for token in test_tokens {
        let sentiment = market_data.get_token_sentiment(token).await?;
        info!("Sentiment for {}: {:?}", token, sentiment);
    }

    Ok(())
} 