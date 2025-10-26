use std::env;

pub struct Config {
    pub gitea_authorization: String,
    pub gitea_base_url: String,
    pub gitea_token: String,
    pub ai_base_url: String,
    pub ai_key: String,
    pub ai_model: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let gitea_authorization = env::var("GITEA_AUTHORIZATION")?;
        let gitea_base_url = env::var("GITEA_BASE_URL")?;
        let gitea_token = env::var("GITEA_TOKEN")?;
        let ai_base_url = env::var("AI_BASE_URL")?;
        let ai_key = env::var("AI_KEY")?;
        let ai_model = env::var("AI_MODEL")?;
        let config = Self {
            gitea_authorization,
            gitea_base_url,
            gitea_token,
            ai_base_url,
            ai_key,
            ai_model,
        };

        Ok(config)
    }
}
