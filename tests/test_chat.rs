use std::error::Error;

use typo_compiler::provider;
use typo_compiler::service::Service;

#[tokio::test]
async fn test_chat() -> Result<(), Box<dyn Error>> {
    let p = provider::read_provider()?;
    let s = Service::new(p);
    let ret = s.post("When do you have time to meet with me".to_string()).await?;

    assert!(!ret.trim().is_empty(), "chat response should not be empty");
    println!("response:\n{ret}");

    Ok(())
}
