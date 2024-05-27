use eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod command;
mod handler;
mod http;

fn init_logging() {
	let fmt_layer = tracing_subscriber::fmt::layer().pretty();
	let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
		.unwrap_or_else(|_| "nixpkgs_discord_tracker=info,warn".into());

	tracing_subscriber::registry()
		.with(fmt_layer)
		.with(env_filter)
		.init();
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().ok();
	init_logging();

	let mut client = client::get().await;
	client.start().await?;

	Ok(())
}
