use solana_sdk::{
    account_info::AccountInfo,
    nonce::State
};

pub struct State<'a> {
    pub account: AccountInfo<'a>,
    // Add other state fields
}

impl<'a> State<'a> {
    pub fn new(account: AccountInfo<'a>) -> Self {
        Self { account }
    }
} 