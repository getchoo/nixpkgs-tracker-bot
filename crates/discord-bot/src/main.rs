#[tokio::main]
async fn main() -> eyre::Result<()> {
	dotenvy::dotenv().ok();
	env_logger::try_init()?;

	let mut client = nixpkgs_tracker_bot::client().await?;
	client.start().await?;

	Ok(())
}
