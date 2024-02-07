#[tokio::main]
async fn main() -> anyhow::Result<()> {
    web::entry().await?;

    Ok(())
}
