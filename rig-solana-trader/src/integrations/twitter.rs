use oauth1::Token;
use reqwest::Client;
use rig_solana_trader::personality::StoicPersonality;

pub struct TwitterClient {
    client: Client,
    personality: StoicPersonality,
}

impl TwitterClient {
    pub fn new(personality: StoicPersonality) -> Self {
        Self {
            client: Client::new(),
            personality,
        }
    }

    pub async fn post_trade(&self, action: &TradeAction, tx_hash: &str) -> Result<()> {
        let tweet = self.personality
            .generate_trade_tweet(action, tx_hash)
            .await?;

        let token = Token::new(
            &std::env::var("TWITTER_API_KEY")?,
            &std::env::var("TWITTER_API_SECRET")?,
        );
        
        let access = Token::new(
            &std::env::var("TWITTER_ACCESS_TOKEN")?,
            &std::env::var("TWITTER_ACCESS_SECRET")?,
        );

        let auth_header = oauth1::authorize("POST", "https://api.twitter.com/2/tweets", &token, Some(&access), None);

        self.client
            .post("https://api.twitter.com/2/tweets")
            .header("Authorization", auth_header)
            .json(&serde_json::json!({ "text": tweet }))
            .send()
            .await?;

        Ok(())
    }
} 