use std::sync::Arc;

use gitea_sdk::Auth;
use gitea_sdk::Client as GiteaClient;

use crate::ai_client::AiClient;
use crate::config::Config;

pub type AppState = Arc<State>;

pub struct State {
    pub gitea_authorization: String,
    pub gitea_client: GiteaClient,
    pub ai_client: AiClient,
}

impl State {
    pub fn new(config: &Config) -> anyhow::Result<AppState> {
        let gitea_client =
            GiteaClient::new(&config.gitea_base_url, Auth::Token(&config.gitea_token));
        let ai_client = AiClient::new(&config.ai_base_url, &config.ai_model, &config.ai_key);
        let state = Self {
            gitea_authorization: config.gitea_authorization.clone(),
            gitea_client,
            ai_client,
        };

        Ok(Arc::new(state))
    }
}
