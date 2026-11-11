use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, ParseKeypairError},
};
use std::str::FromStr;

pub fn load_wallet() -> Result<Keypair, ParseKeypairError> {
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY must be set in .env");
    
    Keypair::from_base58_string(&private_key)
}

pub fn get_public_key(keypair: &Keypair) -> Pubkey {
    keypair.pubkey()
}

pub fn load_keypair() -> Keypair {
    Keypair::new() // Use proper keypair loading in production
} 