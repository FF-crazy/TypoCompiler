use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;

use crate::PROMPT;
use crate::provider::{Provider, ProviderType, Reasoning};

// TODO: support rate control

fn reasoning_to_think_budget(reasoning: &Reasoning, max_tokens: u32) -> u32 {
    let percentage: u32 = match reasoning {
        Reasoning::Low => 15,
        Reasoning::Medium => 25,
        Reasoning::High => 50,
        Reasoning::Xhigh => 75,
    };

    (max_tokens * percentage) / 100
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicRequest {
    model: String,
    system: String,
    max_tokens: u32,
    thinking: AnthropicThinking,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicThinking {
    #[serde(rename = "type")]
    mode: String,
    #[serde(rename = "budget_tokens")]
    think_budget: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIReasonEffort {
    effort: Reasoning,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
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

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub output: Vec<OpenAIOutput>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIOutput {
    #[serde(default)]
    pub content: Vec<OpenAIContent>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIContent {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub annotations: Vec<OpenAIAnnotation>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIAnnotation {}

pub struct Service {
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

    fn build_header(&self) -> Result<HeaderMap, Box<dyn Error>> {
        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match &self.provider.provider_type {
            ProviderType::OpenAI => {
                let auth_value = format!("Bearer {}", self.provider.api_key);
                let header_value = HeaderValue::from_str(&auth_value)
                    .map_err(|e| io::Error::other(format!("invalid authorization header: {e}")))?;
                headers.insert(AUTHORIZATION, header_value);
            }
            ProviderType::Anthropic => {
                let api_key = HeaderValue::from_str(&self.provider.api_key)
                    .map_err(|e| io::Error::other(format!("invalid x-api-key header: {e}")))?;
                headers.insert("x-api-key", api_key);
                headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
            }
        }

        Ok(headers)
    }

    fn build_openai_request(&self, user_input: &str) -> OpenAIRequest {
        OpenAIRequest {
            model: self.provider.model.clone(),
            reasoning: OpenAIReasonEffort {
                effort: self.provider.reasoning.clone(),
            },
            instructions: self.get_instructions(),
            input: vec![OpenAIMessage {
                role: String::from("user"),
                content: user_input.to_string(),
            }],
        }
    }

    fn build_anthropic_request(&self, user_input: &str) -> AnthropicRequest {
        let max_tokens = 65535;

        AnthropicRequest {
            model: self.provider.model.clone(),
            system: self.get_instructions(),
            max_tokens,
            thinking: AnthropicThinking {
                mode: String::from("enabled"),
                think_budget: reasoning_to_think_budget(&self.provider.reasoning, max_tokens),
            },
            messages: vec![AnthropicMessage {
                role: String::from("user"),
                content: user_input.to_string(),
            }],
        }
    }

    fn get_instructions(&self) -> String {
        PROMPT.trim().to_string()
    }

    async fn post_openai_request(
        &self,
        user_input: &str,
    ) -> Result<OpenAIResponse, Box<dyn Error>> {
        let request_body: OpenAIRequest = self.build_openai_request(user_input);
        let headers = self.build_header()?;
        let response = self
            .client
            .post(self.build_request_url())
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(io::Error::other(format!(
                "openai request failed: status={status}, body={body}"
            ))
            .into());
        }

        let response_data: OpenAIResponse = response.json().await?;
        Ok(response_data)
    }

    async fn post_anthropic_request(
        &self,
        user_input: &str,
    ) -> Result<AnthropicResponse, Box<dyn Error>> {
        let request_body = self.build_anthropic_request(user_input);
        let headers = self.build_header()?;

        let response = self
            .client
            .post(self.build_request_url())
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(io::Error::other(format!(
                "anthropic request failed: status={}, body={}",
                status, body
            ))
            .into());
        }

        let response_data: AnthropicResponse = response.json().await?;
        Ok(response_data)
    }

    fn extract_openai(&self, resp: OpenAIResponse) -> String {
        let texts: Vec<String> = resp
            .output
            .into_iter()
            .flat_map(|item| item.content.into_iter())
            .filter(|item| !item.text.trim().is_empty())
            .map(|item| item.text)
            .collect();

        texts.join("\n")
    }

    fn extract_anthropic(&self, resp: AnthropicResponse) -> String {
        let texts: Vec<String> = resp
            .content
            .into_iter()
            .filter(|item| !item.text.trim().is_empty())
            .map(|item| item.text)
            .collect();

        texts.join("\n")
    }

    pub async fn post(&self, user_input: &str) -> Result<String, Box<dyn Error>> {
        match &self.provider.provider_type {
            ProviderType::OpenAI => {
                let resp = self.post_openai_request(user_input).await?;
                Ok(self.extract_openai(resp))
            }
            ProviderType::Anthropic => {
                let resp = self.post_anthropic_request(user_input).await?;
                Ok(self.extract_anthropic(resp))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasoning_to_think_budget_uses_expected_percentages() {
        let max_tokens = 1000;

        assert_eq!(reasoning_to_think_budget(&Reasoning::Low, max_tokens), 150);
        assert_eq!(
            reasoning_to_think_budget(&Reasoning::Medium, max_tokens),
            250
        );
        assert_eq!(reasoning_to_think_budget(&Reasoning::High, max_tokens), 500);
        assert_eq!(
            reasoning_to_think_budget(&Reasoning::Xhigh, max_tokens),
            750
        );
    }
}
