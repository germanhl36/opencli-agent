use super::schema::{AppConfig, ProjectConfig};
use crate::error::OpenCLIError;
use std::path::{Path, PathBuf};

pub struct ConfigLoader {
    app_data_dir: PathBuf,
}

impl ConfigLoader {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self { app_data_dir }
    }

    fn config_path(&self) -> PathBuf {
        self.app_data_dir.join("config.yaml")
    }

    pub fn load(&self) -> Result<AppConfig, OpenCLIError> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(AppConfig::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig =
            serde_yaml::from_str(&content).map_err(|e| OpenCLIError::Config(e.to_string()))?;
        Ok(config)
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), OpenCLIError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content =
            serde_yaml::to_string(config).map_err(|e| OpenCLIError::Config(e.to_string()))?;
        // Atomic write: write to tmp then rename
        let tmp_path = path.with_extension("yaml.tmp");
        std::fs::write(&tmp_path, &content)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn load_project_config(
        &self,
        working_dir: &Path,
    ) -> Result<Option<ProjectConfig>, OpenCLIError> {
        let project_config_path = working_dir.join(".opencli.yaml");
        if !project_config_path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&project_config_path)?;
        let config: ProjectConfig = serde_yaml::from_str(&content)
            .map_err(|e| OpenCLIError::InvalidProjectConfig(e.to_string()))?;
        Ok(Some(config))
    }

    pub fn merge_configs(
        &self,
        app_config: &AppConfig,
        project_config: &ProjectConfig,
    ) -> AppConfig {
        let mut merged = app_config.clone();
        if let Some(model) = &project_config.model_override {
            merged.active_model = model.clone();
        }
        if let Some(provider) = &project_config.provider_override {
            merged.active_provider = provider.clone();
        }
        if let Some(sandbox) = project_config.sandbox_enabled {
            merged.sandbox_enabled = sandbox;
        }
        if let Some(image) = &project_config.sandbox_image {
            merged.sandbox_image = Some(image.clone());
        }
        merged
    }

    pub fn validate(config: &AppConfig) -> Result<(), OpenCLIError> {
        if config.active_provider.is_empty() {
            return Err(OpenCLIError::Config(
                "activeProvider cannot be empty".to_string(),
            ));
        }
        if config.active_model.is_empty() {
            return Err(OpenCLIError::Config(
                "activeModel cannot be empty".to_string(),
            ));
        }
        if config.font_size < 8 || config.font_size > 72 {
            return Err(OpenCLIError::Config(
                "fontSize must be between 8 and 72".to_string(),
            ));
        }
        if config.command_timeout_s < 1 || config.command_timeout_s > 3600 {
            return Err(OpenCLIError::Config(
                "commandTimeoutS must be between 1 and 3600".to_string(),
            ));
        }
        Ok(())
    }
}
