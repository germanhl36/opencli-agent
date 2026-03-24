use std::sync::Arc;
use crate::error::OpenCLIError;
use crate::llm::provider::LLMProvider;
use crate::llm::providers::{
    ollama::OllamaProvider,
    openrouter::OpenRouterProvider,
    huggingface::HuggingFaceProvider,
    custom::CustomProvider,
};

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub provider_name: Option<String>,
}

pub fn create_provider(config: ProviderConfig) -> Result<Arc<dyn LLMProvider>, OpenCLIError> {
    match config.provider_type.as_str() {
        "ollama" => Ok(Arc::new(OllamaProvider::new(config.base_url))),
        "openrouter" => {
            let key = config.api_key.ok_or_else(|| {
                OpenCLIError::Config("OpenRouter requires an API key".to_string())
            })?;
            Ok(Arc::new(OpenRouterProvider::new(key)))
        }
        "huggingface" => {
            let key = config.api_key.ok_or_else(|| {
                OpenCLIError::Config("HuggingFace requires an API key".to_string())
            })?;
            Ok(Arc::new(HuggingFaceProvider::new(key, config.base_url)))
        }
        "custom" => {
            let url = config.base_url.ok_or_else(|| {
                OpenCLIError::Config("Custom provider requires a base URL".to_string())
            })?;
            Ok(Arc::new(CustomProvider::new(url, config.api_key, config.provider_name)))
        }
        other => Err(OpenCLIError::Llm(format!("Unknown provider type: {}", other))),
    }
}

pub fn provider_from_app_config(
    app_config: &crate::config::schema::AppConfig,
    api_key: Option<String>,
) -> Result<Arc<dyn LLMProvider>, OpenCLIError> {
    create_provider(ProviderConfig {
        provider_type: app_config.active_provider.clone(),
        base_url: None,
        api_key,
        provider_name: Some(app_config.active_provider.clone()),
    })
}
