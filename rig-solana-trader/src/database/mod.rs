pub mod sync;

use anyhow::Result;
use mongodb::{
    bson::{doc, Document},
    Client, Collection, Database as MongoDatabase,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    client: Client,
    db: MongoDatabase,
    collections: Arc<Mutex<Vec<String>>>,
}

impl Database {
    pub async fn new(uri: &str, db_name: &str) -> Result<Self> {
        let client = Client::with_uri_str(uri).await?;
        let db = client.database(db_name);
        
        Ok(Self {
            client,
            db,
            collections: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn get_collection<T>(&self, name: &str) -> Result<Collection<T>> 
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    {
        let mut collections = self.collections.lock().await;
        if !collections.contains(&name.to_string()) {
            collections.push(name.to_string());
        }
        
        Ok(self.db.collection(name))
    }

    pub async fn insert_one<T>(&self, collection: &str, document: T) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    {
        let collection = self.get_collection::<T>(collection).await?;
        collection.insert_one(document, None).await?;
        Ok(())
    }

    pub async fn find_one<T>(&self, collection: &str, filter: Document) -> Result<Option<T>>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    {
        let collection = self.get_collection::<T>(collection).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    pub async fn update_one<T>(
        &self,
        collection: &str,
        filter: Document,
        update: Document,
    ) -> Result<()>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync,
    {
        let collection = self.get_collection::<T>(collection).await?;
        collection.update_one(filter, update, None).await?;
        Ok(())
    }
} 