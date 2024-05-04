use anyhow::Result;
use morph::Instruction;

pub mod client;
pub mod images;
pub mod morph;
pub mod stdin;

#[tokio::main]
async fn main() -> Result<()> {
    let instruction: Instruction =
        serde_json::from_str(&stdin::read_stdin()).expect("Invalid JSON");
    morph::morph(instruction).await;
    Ok(())
}
