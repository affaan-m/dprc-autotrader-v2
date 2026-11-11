use crate::market_data::EnhancedTokenMetadata;
use crate::strategy::{TradingDecision, ExecutionParams};
use anyhow::Result;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ExecutionEngine {
    max_slippage: f64,
    active_orders: HashMap<String, ActiveOrder>,
    execution_history: Vec<ExecutionRecord>,
    last_execution: Option<Instant>,
    min_execution_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct ActiveOrder {
    pub token_address: String,
    pub order_type: OrderType,
    pub size_in_sol: f64,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profits: Vec<f64>,
    pub filled_amount: f64,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub token_address: String,
    pub order_type: OrderType,
    pub size_in_sol: f64,
    pub execution_price: f64,
    pub slippage: f64,
    pub timestamp: DateTime<Utc>,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled(f64),
    Filled,
    Cancelled,
    Failed(String),
}

impl ExecutionEngine {
    pub fn new(max_slippage: f64) -> Self {
        info!("Initializing ExecutionEngine with max_slippage: {}", max_slippage);
        Self {
            max_slippage,
            active_orders: HashMap::new(),
            execution_history: Vec::new(),
            last_execution: None,
            min_execution_interval: Duration::from_secs(300), // 5 minutes between trades
        }
    }

    pub async fn execute_trade(
        &mut self,
        decision: &TradingDecision,
        token: &EnhancedTokenMetadata,
    ) -> Result<ExecutionRecord> {
        // Check execution cooldown
        if let Some(last_exec) = self.last_execution {
            let elapsed = last_exec.elapsed();
            if elapsed < self.min_execution_interval {
                let wait_time = self.min_execution_interval - elapsed;
                warn!("Trade execution cooldown in effect. Must wait {:?} before next trade", wait_time);
                return Err(anyhow::anyhow!("Trade execution cooldown in effect"));
            }
        }

        info!("Executing trade for token: {} ({:?})", token.symbol, decision.action);
        debug!("Trade details - Size: {} SOL, Risk Score: {}", decision.size_in_sol, decision.risk_score);

        // 1. Validate execution parameters
        self.validate_execution_params(&decision.execution_params)
            .map_err(|e| {
                error!("Execution parameter validation failed: {}", e);
                e
            })?;

        // 2. Check for existing orders
        if let Some(active_order) = self.active_orders.get(&decision.token_address) {
            debug!("Found existing order for token: {:?}", active_order);
            self.handle_existing_order(active_order)
                .map_err(|e| {
                    error!("Failed to handle existing order: {}", e);
                    e
                })?;
        }

        // 3. Prepare order parameters
        let order = self.prepare_order(decision, token);
        debug!("Prepared order: {:?}", order);

        // 4. Execute the order
        let execution_record = self.submit_order(order).await
            .map_err(|e| {
                error!("Order submission failed: {}", e);
                e
            })?;

        // 5. Update order tracking
        self.update_order_tracking(&execution_record);
        info!("Trade executed successfully: {:?}", execution_record);

        // Update last execution time
        self.last_execution = Some(Instant::now());

        Ok(execution_record)
    }

    fn validate_execution_params(&self, params: &ExecutionParams) -> Result<()> {
        debug!("Validating execution parameters: {:?}", params);

        // Validate slippage
        if params.max_slippage > self.max_slippage {
            warn!("Slippage {} exceeds maximum allowed {}", params.max_slippage, self.max_slippage);
            return Err(anyhow::anyhow!("Slippage exceeds maximum allowed"));
        }

        // Validate stop loss
        if params.stop_loss <= 0.0 || params.stop_loss > 0.5 {
            warn!("Invalid stop loss percentage: {}", params.stop_loss);
            return Err(anyhow::anyhow!("Invalid stop loss percentage"));
        }

        // Validate take profit levels
        if params.take_profit.is_empty() {
            warn!("No take profit levels specified");
            return Err(anyhow::anyhow!("No take profit levels specified"));
        }

        for (i, tp) in params.take_profit.iter().enumerate() {
            if *tp <= params.stop_loss {
                warn!("Take profit level {} ({}) must be greater than stop loss ({})", i, tp, params.stop_loss);
                return Err(anyhow::anyhow!("Take profit must be greater than stop loss"));
            }
        }

        debug!("Execution parameters validated successfully");
        Ok(())
    }

    fn handle_existing_order(&self, order: &ActiveOrder) -> Result<()> {
        match order.status {
            OrderStatus::Pending | OrderStatus::PartiallyFilled(_) => {
                warn!("Active order exists for token {}: {:?}", order.token_address, order.status);
                Err(anyhow::anyhow!("Active order exists for this token"))
            }
            _ => {
                debug!("No conflicting active order found");
                Ok(())
            }
        }
    }

    fn prepare_order(&self, decision: &TradingDecision, token: &EnhancedTokenMetadata) -> ActiveOrder {
        debug!("Preparing order for token: {}", token.symbol);
        
        let order = ActiveOrder {
            token_address: decision.token_address.clone(),
            order_type: match decision.execution_params.entry_type.as_str() {
                "Market" => OrderType::Market,
                "Limit" => OrderType::Limit,
                _ => OrderType::Market,
            },
            size_in_sol: decision.size_in_sol,
            entry_price: token.price_sol,
            stop_loss: token.price_sol * (1.0 - decision.execution_params.stop_loss),
            take_profits: decision.execution_params.take_profit.iter()
                .map(|tp| token.price_sol * (1.0 + tp))
                .collect(),
            filled_amount: 0.0,
            status: OrderStatus::Pending,
            timestamp: Utc::now(),
        };

        debug!("Order prepared: {:?}", order);
        order
    }

    async fn submit_order(&self, order: ActiveOrder) -> Result<ExecutionRecord> {
        info!("Submitting order: {:?}", order);
        
        // TODO: Implement actual order submission through Jupiter DEX
        // For now, simulate a successful market order
        let record = ExecutionRecord {
            token_address: order.token_address,
            order_type: order.order_type,
            size_in_sol: order.size_in_sol,
            execution_price: order.entry_price,
            slippage: 0.001, // 0.1% simulated slippage
            timestamp: Utc::now(),
            tx_signature: Some("simulated_tx_signature".to_string()),
        };

        info!("Order submitted successfully: {:?}", record);
        Ok(record)
    }

    fn update_order_tracking(&mut self, record: &ExecutionRecord) {
        debug!("Updating order tracking for token: {}", record.token_address);
        self.execution_history.push(record.clone());
        self.active_orders.remove(&record.token_address);
        debug!("Order tracking updated. Active orders: {}", self.active_orders.len());
    }

    pub fn get_active_orders(&self) -> &HashMap<String, ActiveOrder> {
        &self.active_orders
    }

    pub fn get_execution_history(&self) -> &Vec<ExecutionRecord> {
        &self.execution_history
    }
} 