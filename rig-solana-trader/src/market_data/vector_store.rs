use rig_core::{
    embeddings::EmbeddingsBuilder,
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStoreIndex},
    Embed,
};
use rig_mongodb::MongoDBVectorStore;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, debug};
use crate::database::DatabaseClient;
use mongodb::bson;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    pub token_address: String,
    pub sentiment_score: f64,
    pub technical_score: f64,
    pub risk_score: f64
}

pub struct TokenVectorStore {
    memory_index: VectorStoreIndex<InMemoryVectorStore<TokenAnalysis>>,
    mongo_store: MongoDBVectorStore<TokenAnalysis>,
}

impl TokenVectorStore {
    pub async fn new(openai_api_key: &str, db_client: DatabaseClient) -> Result<Self> {
        let client = Client::new(openai_api_key);
        let embedding_model = client.embedding_model(TEXT_EMBEDDING_ADA_002);
        
        // Initialize in-memory store
        let memory_store = InMemoryVectorStore::default();
        let memory_index = memory_store.index(embedding_model.clone());
        
        // Initialize MongoDB store
        let mongo_store = MongoDBVectorStore::new(
            db_client.get_database(),
            "token_analysis",
            embedding_model,
        );
        
        Ok(Self {
            memory_index,
            mongo_store,
        })
    }

    pub async fn add_token_analysis(&mut self, analysis: TokenAnalysis) -> Result<()> {
        debug!("Adding token analysis for {}", analysis.token_address);
        
        // Generate embedding
        let embeddings = EmbeddingsBuilder::new(self.memory_index.model().clone())
            .documents(vec![analysis.clone()])?
            .build()
            .await?;
            
        // Save to in-memory store
        self.memory_index.store_mut().extend(embeddings.clone());
        
        // Save to MongoDB store
        self.mongo_store.add_document(analysis, embeddings.embeddings[0].clone()).await?;
        
        Ok(())
    }

    pub async fn find_similar_tokens(&self, query: &str, limit: usize) -> Result<Vec<(f32, String, TokenAnalysis)>> {
        debug!("Searching for tokens similar to query: {}", query);
        
        // Try MongoDB store first
        if let Ok(results) = self.mongo_store.search(query, limit).await {
            info!("Found {} similar tokens in MongoDB", results.len());
            return Ok(results);
        }
        
        // Fall back to in-memory store
        let results = self.memory_index
            .top_n::<TokenAnalysis>(query, limit)
            .await?;
            
        info!("Found {} similar tokens in memory", results.len());
        Ok(results)
    }

    pub async fn get_token_sentiment(&self, token_address: &str) -> Result<Option<String>> {
        // Try MongoDB store first
        if let Ok(Some(analysis)) = self.mongo_store.get_document(token_address).await {
            return Ok(Some(format!("Sentiment Score: {:.2}", analysis.sentiment_score)));
        }
        
        // Fall back to vector search
        let results = self.memory_index
            .top_n::<TokenAnalysis>(&format!("sentiment analysis for token {}", token_address), 1)
            .await?;
            
        Ok(results.first().map(|(_, _, analysis)| format!("Sentiment Score: {:.2}", analysis.sentiment_score)))
    }

    pub async fn store_embedding(&self, data_type: &str, vector: Vec<f32>) {
        self.mongo_store.client.insert_one(
            doc! { 
                "type": data_type,
                "vector": bson::to_bson(&vector).unwrap(),
                "timestamp": Utc::now()
            }, 
            None
        ).await.unwrap();
    }
} 