//! Rig Solana Trader is an autonomous trading bot for the Solana blockchain.
//! 
//! # Overview
//! This bot uses LLM-powered analysis to make trading decisions on Solana tokens.
//! It combines market data from multiple sources, technical analysis, and stoic
//! principles to execute trades through Jupiter DEX.
//!
//! # Key Features
//! - Real-time market data aggregation from BirdEye API
//! - LLM-powered trading analysis with stoic personality
//! - Automated trade execution via Jupiter DEX
//! - Position tracking and portfolio management
//! - Twitter integration for trade announcements
//! - MongoDB persistence for market data and positions
//!
//! # Architecture
//! The bot consists of several key components:
//! - `market_data`: Interfaces with BirdEye API for token data
//! - `strategy`: Implements trading logic and LLM analysis
//! - `personality`: Handles stoic tweet generation
//! - `dex`: Manages trade execution via Jupiter
//! - `database`: Handles MongoDB persistence
//! - `twitter`: Manages Twitter interactions

pub mod database;
pub mod dex;
pub mod market_data;
pub mod personality;
pub mod strategy;
pub mod twitter;

// ... rest of the file ... 