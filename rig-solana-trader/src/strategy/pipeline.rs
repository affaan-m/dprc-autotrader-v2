use rig::pipeline::{Op, Pipeline, TryOp};
use anyhow::Result;
use crate::{
    market_data::{MarketDataProvider, TokenAnalysis},
    strategy::{TradingStrategy, TradingDecision},
    execution::ExecutionEngine,
};
use tracing::{info, debug};

pub struct MarketAnalysisOp {
    market_data: MarketDataProvider,
}

impl MarketAnalysisOp {
    pub fn new(market_data: MarketDataProvider) -> Self {
        Self { market_data }
    }
}

impl TryOp<String, TokenAnalysis> for MarketAnalysisOp {
    async fn try_run(&self, token_address: String) -> Result<TokenAnalysis> {
        debug!("Running market analysis for token {}", token_address);
        self.market_data.analyze_token(&token_address).await?;
        let analysis = self.market_data.get_token_analysis(&token_address).await?
            .ok_or_else(|| anyhow::anyhow!("No analysis found for token"))?;
        Ok(analysis)
    }
}

pub struct StrategyOp {
    strategy: TradingStrategy,
}

impl StrategyOp {
    pub fn new(strategy: TradingStrategy) -> Self {
        Self { strategy }
    }
}

impl TryOp<TokenAnalysis, TradingDecision> for StrategyOp {
    async fn try_run(&self, analysis: TokenAnalysis) -> Result<TradingDecision> {
        debug!("Generating trading decision for token {}", analysis.symbol);
        self.strategy.generate_decision(&analysis).await
    }
}

pub struct ExecutionOp {
    engine: ExecutionEngine,
}

impl ExecutionOp {
    pub fn new(engine: ExecutionEngine) -> Self {
        Self { engine }
    }
}

impl TryOp<TradingDecision, String> for ExecutionOp {
    async fn try_run(&self, decision: TradingDecision) -> Result<String> {
        debug!("Executing trading decision: {:?}", decision);
        let record = self.engine.execute_trade(&decision).await?;
        Ok(record.tx_signature.unwrap_or_default())
    }
}

pub struct TradingPipeline {
    pipeline: Pipeline<String, String>,
}

impl TradingPipeline {
    pub fn new(market_data: MarketDataProvider, strategy: TradingStrategy, execution: ExecutionEngine) -> Self {
        let pipeline = Pipeline::new()
            .add_try_op(MarketAnalysisOp::new(market_data))
            .add_try_op(StrategyOp::new(strategy))
            .add_try_op(ExecutionOp::new(execution));
            
        Self { pipeline }
    }
    
    pub async fn execute_trade(&self, token_address: String) -> Result<String> {
        info!("Starting trading pipeline for token {}", token_address);
        self.pipeline.try_run(token_address).await
    }
} 