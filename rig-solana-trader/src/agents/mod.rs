use rig_core::{
    agent::{Agent, AgentBuilder},
    chat::{Chat, CompletionModel, Message, PromptError},
    providers::openai::Client as OpenAIClient,
    Result,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use rig_solana_trader::market_data::{MarketData, TokenMetadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketAnalysis {
    pub token: TokenMetadata,
    pub sentiment_score: f64,
    pub risk_score: f64,
    pub recommendation: String,
    pub reasoning: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RiskAssessment {
    pub token: TokenMetadata,
    pub liquidity_risk: f64,
    pub volatility_risk: f64,
    pub market_risk: f64,
    pub overall_risk: f64,
    pub risk_factors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExecutionPlan {
    pub token: TokenMetadata,
    pub action: String,
    pub size: f64,
    pub target_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub reasoning: String,
}

pub struct TradingAgentSystem {
    market_analyst: Agent,
    risk_manager: Agent,
    execution_specialist: Agent,
}

impl TradingAgentSystem {
    pub async fn new(openai_client: OpenAIClient) -> Result<Self> {
        // Initialize market analyst agent
        let market_analyst = AgentBuilder::new()
            .name("Market Analyst")
            .description("Analyzes market data and provides trading recommendations")
            .model(CompletionModel::GPT4)
            .client(openai_client.clone())
            .build()?;

        // Initialize risk manager agent
        let risk_manager = AgentBuilder::new()
            .name("Risk Manager")
            .description("Assesses trading risks and provides risk management recommendations")
            .model(CompletionModel::GPT4)
            .client(openai_client.clone())
            .build()?;

        // Initialize execution specialist agent
        let execution_specialist = AgentBuilder::new()
            .name("Execution Specialist")
            .description("Plans and optimizes trade execution")
            .model(CompletionModel::GPT4)
            .client(openai_client)
            .build()?;

        Ok(Self {
            market_analyst,
            risk_manager,
            execution_specialist,
        })
    }

    pub async fn analyze_market(&self, market_data: &MarketData) -> Result<MarketAnalysis> {
        debug!("Analyzing market data for {}", market_data.token.symbol);

        let prompt = format!(
            "Analyze the following market data and provide a trading recommendation:\n\
            Token: {} ({})\n\
            Price: ${}\n\
            24h Volume: ${}\n\
            Market Cap: ${}\n\
            Social Sentiment: {}\n\
            Technical Indicators:\n\
            - RSI (14): {}\n\
            - MACD: {}\n\
            - MA50: {}\n\
            - MA200: {}\n\
            \n\
            Provide your analysis in JSON format with the following fields:\n\
            - sentiment_score: A score between 0 and 1\n\
            - risk_score: A score between 0 and 1\n\
            - recommendation: A brief trading recommendation\n\
            - reasoning: Your detailed reasoning",
            market_data.token.symbol,
            market_data.token.address,
            market_data.token.price_usd.unwrap_or_default(),
            market_data.token.volume_24h.unwrap_or_default(),
            market_data.token.market_cap.unwrap_or_default(),
            market_data.social_sentiment.unwrap_or_default(),
            market_data.technical_indicators.rsi_14.unwrap_or_default(),
            market_data.technical_indicators.macd.unwrap_or_default(),
            market_data.technical_indicators.ma_50.unwrap_or_default(),
            market_data.technical_indicators.ma_200.unwrap_or_default(),
        );

        let response = self.market_analyst
            .chat(&[Message::user(&prompt)])
            .await?;

        let analysis: MarketAnalysis = serde_json::from_str(&response.content)?;
        Ok(analysis)
    }

    pub async fn assess_risk(&self, market_data: &MarketData, analysis: &MarketAnalysis) -> Result<RiskAssessment> {
        debug!("Assessing risk for {}", market_data.token.symbol);

        let prompt = format!(
            "Assess the trading risks for the following token based on market data and analysis:\n\
            Token: {} ({})\n\
            Market Analysis:\n\
            - Sentiment Score: {}\n\
            - Risk Score: {}\n\
            - Recommendation: {}\n\
            \n\
            Market Data:\n\
            - Price: ${}\n\
            - 24h Volume: ${}\n\
            - Market Cap: ${}\n\
            \n\
            Provide your assessment in JSON format with the following fields:\n\
            - liquidity_risk: A score between 0 and 1\n\
            - volatility_risk: A score between 0 and 1\n\
            - market_risk: A score between 0 and 1\n\
            - overall_risk: A weighted average of the above risks\n\
            - risk_factors: An array of specific risk factors identified",
            market_data.token.symbol,
            market_data.token.address,
            analysis.sentiment_score,
            analysis.risk_score,
            analysis.recommendation,
            market_data.token.price_usd.unwrap_or_default(),
            market_data.token.volume_24h.unwrap_or_default(),
            market_data.token.market_cap.unwrap_or_default(),
        );

        let response = self.risk_manager
            .chat(&[Message::user(&prompt)])
            .await?;

        let assessment: RiskAssessment = serde_json::from_str(&response.content)?;
        Ok(assessment)
    }

    pub async fn plan_execution(
        &self,
        market_data: &MarketData,
        analysis: &MarketAnalysis,
        risk: &RiskAssessment,
    ) -> Result<ExecutionPlan> {
        debug!("Planning execution for {}", market_data.token.symbol);

        let prompt = format!(
            "Plan the execution of a trade based on the following analysis and risk assessment:\n\
            Token: {} ({})\n\
            Current Price: ${}\n\
            \n\
            Market Analysis:\n\
            - Sentiment Score: {}\n\
            - Risk Score: {}\n\
            - Recommendation: {}\n\
            \n\
            Risk Assessment:\n\
            - Overall Risk: {}\n\
            - Risk Factors: {}\n\
            \n\
            Provide your execution plan in JSON format with the following fields:\n\
            - action: 'BUY' or 'SELL'\n\
            - size: Position size in SOL\n\
            - target_price: Entry price target\n\
            - stop_loss: Stop loss price\n\
            - take_profit: Take profit price\n\
            - reasoning: Detailed reasoning for the execution plan",
            market_data.token.symbol,
            market_data.token.address,
            market_data.token.price_usd.unwrap_or_default(),
            analysis.sentiment_score,
            analysis.risk_score,
            analysis.recommendation,
            risk.overall_risk,
            risk.risk_factors.join(", "),
        );

        let response = self.execution_specialist
            .chat(&[Message::user(&prompt)])
            .await?;

        let plan: ExecutionPlan = serde_json::from_str(&response.content)?;
        Ok(plan)
    }
} 