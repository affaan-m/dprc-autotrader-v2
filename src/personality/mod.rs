pub async fn generate_trade_tweet(&self, agent: &CompletionModel, trade_details: String) -> Result<String> {
    info!("Generating trade tweet with details: {}", trade_details);
    
    let prompt = format!(
        "{}\n\nPlease generate a tweet about this trade that:\n1. Is concise and professional\n2. Includes key metrics (amount, price, volume)\n3. Includes contract address and tx link\n4. Ends with stoic analysis based on market indicators\n5. Stays under 280 characters",
        trade_details
    );

    let tweet = agent.complete(&prompt).await?;
    info!("Generated tweet: {}", tweet);
    
    Ok(tweet)
} 