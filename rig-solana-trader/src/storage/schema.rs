use mongodb::bson::{doc, oid::ObjectId, DateTime, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentData {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub agent_type: String,
    pub vector_embedding: Vec<f32>,
    pub metadata: Document,
    pub timestamp: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeExecution {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tx_hash: String,
    pub mint_address: String,
    pub amount: f64,
    pub risk_assessment: f64,
    pub vector_embedding: Vec<f32>,
    pub timestamp: DateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketAnalysis {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub market_cap: f64,
    pub liquidity_ratio: f64,
    pub volume_analysis: Document,
    pub vector_embedding: Vec<f32>,
    pub timestamp: DateTime,
} 