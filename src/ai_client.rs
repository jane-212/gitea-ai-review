use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};

use crate::error::{ApiError, Result};

pub struct AiClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl AiClient {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        let ai_config = OpenAIConfig::new()
            .with_api_base(base_url.into())
            .with_api_key(key.into());
        let client = Client::with_config(ai_config);
        let ai_client = Self {
            client,
            model: model.into(),
        };

        ai_client
    }

    pub async fn chat(&self, message: impl AsRef<str>) -> Result<String> {
        let message = message.as_ref();

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(message)
                    .build()?
                    .into(),
            ])
            .build()?;
        let response = self.client.chat().create(request).await?;

        let text = response
            .choices
            .into_iter()
            .flat_map(|choice| choice.message.content)
            .next()
            .ok_or(ApiError::NoResponse)?;

        Ok(text)
    }
}
