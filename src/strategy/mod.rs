#[instrument(skip(self))]
pub async fn analyze_trading_opportunity(&self, prompt: String, sol_balance: f64) -> Result<String> {
    info!("Analyzing trading opportunity with prompt: {}", prompt);
    
    // Format the prompt with market analysis requirements
    let formatted_prompt = format!(
        "{}\n\nPlease analyze this trading opportunity and provide a recommendation in the following JSON format:\n{{
            \"action\": \"Buy|Sell|Hold\",
            \"token_address\": \"string\",
            \"amount_in_sol\": number,
            \"reasoning\": \"string\",
            \"confidence\": number (0.0-1.0),
            \"risk_assessment\": \"string\",
            \"market_analysis\": {{
                \"volume_analysis\": \"string\",
                \"price_trend\": \"string\",
                \"liquidity_assessment\": \"string\",
                \"momentum_indicators\": \"string\"
            }}
        }}\n\nAvailable SOL balance: {} SOL", 
        prompt,
        sol_balance
    );

    // Get analysis from LLM
    let analysis = self.agent.complete(&formatted_prompt).await?;
    
    info!("Received analysis from LLM");
    Ok(analysis)
} 