impl TwitterClient {
    pub fn new() -> Self {
        TwitterClient {
            api_key: std::env::var("TWITTER_API_KEY").unwrap(),
            api_secret: std::env::var("TWITTER_API_SECRET").unwrap(),
            access_token: std::env::var("TWITTER_ACCESS_TOKEN").unwrap(),
            access_secret: std::env::var("TWITTER_ACCESS_SECRET").unwrap(),
        }
    }
} 