use rig::loaders::{FileLoader, PDFLoader};
use anyhow::Result;
use tracing::debug;
use std::path::Path;

pub struct MarketDataLoader {
    file_loader: FileLoader,
    pdf_loader: PDFLoader,
}

impl MarketDataLoader {
    pub fn new() -> Self {
        Self {
            file_loader: FileLoader::new(),
            pdf_loader: PDFLoader::new(),
        }
    }

    pub async fn load_market_report(&self, path: impl AsRef<Path>) -> Result<String> {
        debug!("Loading market report from {:?}", path.as_ref());
        
        let content = if path.as_ref().extension().map_or(false, |ext| ext == "pdf") {
            self.pdf_loader.load(path).await?
        } else {
            self.file_loader.load(path).await?
        };
        
        Ok(content)
    }

    pub async fn load_token_whitepaper(&self, path: impl AsRef<Path>) -> Result<String> {
        debug!("Loading token whitepaper from {:?}", path.as_ref());
        self.pdf_loader.load(path).await
    }

    pub async fn load_technical_analysis(&self, path: impl AsRef<Path>) -> Result<String> {
        debug!("Loading technical analysis from {:?}", path.as_ref());
        self.file_loader.load(path).await
    }
} 