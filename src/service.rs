use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::{Provider, ProviderType, Reasoning};

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u16,
    messages: Vec<AnthropicMessage>,
}


#[derive(Debug, Serialize, Deserialize)]
struct OpenAIReasonEffort {
  effort: Reasoning,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
  role: String,
  content: String
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    reasoning: OpenAIReasonEffort,
    instructions: String,
    input: Vec<OpenAIMessage>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<AnthropicContent>,
    pub model: String,
    pub stop_reason: String,
    pub usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub type OpenAIResponses = Vec<OpenAIResponse>;

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<OpenAIContent>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
    pub annotations: Vec<OpenAIAnnotation>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIAnnotation {}

struct Service {
    client: Client,
    provider: Provider,
}

impl Service {
    pub fn new(provider: Provider) -> Self {
        Service {
            client: Client::new(),
            provider,
        }
    }

    fn build_request_url(&self) -> String {
        match &self.provider.provider_type {
            ProviderType::OpenAI => format!("{}/v1/responses", self.provider.base_url),
            ProviderType::Anthropic => format!("{}/v1/messages", self.provider.base_url),
        }
    }

    fn build_header(&self) -> Vec<String> {
        let mut header = vec![String::from("Content-Type: application/json")];
        match &self.provider.provider_type {
            ProviderType::OpenAI => {
                header.push(format!("Authorization: Bearer {}", self.provider.api_key));
            }
            ProviderType::Anthropic => {
                header.push(format!("x-api-key: {}", self.provider.api_key));
                header.push(String::from("anthropic-version: 2023-06-01"));
            }
        }
        header
    }

}
