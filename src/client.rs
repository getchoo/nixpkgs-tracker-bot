use std::sync::Arc;

use crate::{
	handler::Handler,
	http::{self, HttpClientExt},
};

use eyre::Result;
use serenity::prelude::{Client, GatewayIntents, TypeMapKey};
use tracing::trace;

/// Container for [http::Client]
pub struct SharedClient;

impl TypeMapKey for SharedClient {
	type Value = Arc<http::Client>;
}

/// Fetch our bot token
fn token() -> Result<String> {
	let token = std::env::var("DISCORD_BOT_TOKEN")?;
	Ok(token)
}

/// Create our client
#[tracing::instrument]
pub async fn get() -> Client {
	let token = token().expect("Couldn't find token in environment! Is DISCORD_BOT_TOKEN set?");

	let intents = GatewayIntents::default();
	trace!("Creating client");
	let client = Client::builder(token, intents)
		.event_handler(Handler)
		.await
		.expect("Couldn't create a client!");

	// add state stuff
	{
		let mut data = client.data.write().await;
		trace!("Creating HTTP client");
		let http_client = <http::Client as HttpClientExt>::default();
		trace!("Inserting HTTP client into Discord client");
		data.insert::<SharedClient>(Arc::new(http_client))
	}

	let shard_manager = client.shard_manager.clone();

	// gracefully shutdown on ctrl+c
	tokio::spawn(async move {
		#[cfg(target_family = "unix")]
		tokio::signal::ctrl_c()
			.await
			.expect("Couldn't registrl ctrl+c handler!");
		shard_manager.shutdown_all().await;
	});

	client
}
