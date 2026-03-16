use typo_compiler::provider::{Provider, ProviderType, Reasoning, read_provider};

#[test]
fn parse_provider_toml_success() {
    let raw = r#"
base_url = "https://api.openai.com"
type = "OpenAI"
api_key = "test-key"
model = "gpt-5"
reasoning = "low"
api_rate = 6
"#;

    let provider: Provider = toml::from_str(raw).expect("provider TOML should parse");

    assert_eq!(provider.base_url, "https://api.openai.com");
    assert_eq!(provider.model, "gpt-5");
    assert!(matches!(provider.provider_type, ProviderType::OpenAI));
    assert!(matches!(provider.reasoning, Reasoning::Low));
}

#[test]
fn parse_provider_toml_invalid_enum_fails() {
    let raw = r#"
base_url = "https://api.openai.com"
type = "UnknownProvider"
api_key = "test-key"
model = "gpt-5"
reasoning = "low"
api_rate = 6
"#;

    let parsed = toml::from_str::<Provider>(raw);
    assert!(parsed.is_err());
}

#[test]
#[ignore]
fn test_load() {
    let result = read_provider();
    assert!(result.is_ok(), "read_provider should load provider.toml");

    let provider = result.unwrap();
    assert!(!provider.base_url.is_empty());
    assert!(!provider.model.is_empty());
}
