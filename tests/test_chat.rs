use std::error::Error;

use typo_compiler::provider;
use typo_compiler::service::Service;

#[tokio::test]
#[ignore]
async fn test_chat() -> Result<(), Box<dyn Error>> {
    let p = provider::read_provider()?;
    let s = Service::new(p);
    let ret = s.post("I hate apple vrey much").await?;

    assert!(!ret.trim().is_empty(), "chat response should not be empty");
    println!("response:\n{ret}");

    Ok(())
}
