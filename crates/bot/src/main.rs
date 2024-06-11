#[tokio::main]
async fn main() -> Result<(), bot_client::Error> {
	dotenvy::dotenv().ok();
	env_logger::try_init()?;

	let mut client = bot_client::get().await?;
	client.start().await?;

	Ok(())
}
