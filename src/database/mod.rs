use mongodb::{bson::doc, Collection, Database};
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;

pub mod positions;
pub mod sync;

pub trait DatabaseExt {
    async fn insert_one<T: Serialize>(&self, collection_name: &str, document: &T) -> Result<()>;
}

impl DatabaseExt for Arc<Database> {
    async fn insert_one<T: Serialize>(&self, collection_name: &str, document: &T) -> Result<()> {
        let collection = self.collection(collection_name);
        collection.insert_one(document, None).await?;
        Ok(())
    }
} 