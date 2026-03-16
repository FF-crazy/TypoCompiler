use serde::{Deserialize, Serialize};
use toml;

use std::fmt;
use std::{fs::read_to_string, io};

use crate::PATH;

#[derive(Deserialize, Debug)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Reasoning {
    Low,
    Medium,
    High,
    Xhigh,
}


#[derive(Deserialize, Debug)]
pub struct Provider {
    pub base_url: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_key: String,
    pub model: String,
    pub reasoning: Reasoning,
    pub api_rate: i32,
}

#[derive(Debug)]
pub enum ProviderError {
    ConfigNotFound { path: String, source: io::Error },
    ConfigReadFailed { path: String, source: io::Error },
    ConfigParseFailed(toml::de::Error),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound { path, .. } => write!(f, "provider config not found: {path}"),
            Self::ConfigReadFailed { path, .. } => {
                write!(f, "failed to read provider config: {path}")
            }
            Self::ConfigParseFailed(_) => write!(f, "failed to parse provider config (TOML)"),
        }
    }
}

impl std::error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConfigNotFound { source, .. } => Some(source),
            Self::ConfigReadFailed { source, .. } => Some(source),
            Self::ConfigParseFailed(source) => Some(source),
        }
    }
}

pub fn read_provider() -> Result<Provider, ProviderError> {
    let file = read_to_string(PATH).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            ProviderError::ConfigNotFound {
                path: PATH.to_string(),
                source: error,
            }
        } else {
            ProviderError::ConfigReadFailed {
                path: PATH.to_string(),
                source: error,
            }
        }
    })?;

    let provider: Provider = toml::from_str(&file).map_err(ProviderError::ConfigParseFailed)?;
    Ok(provider)
}
