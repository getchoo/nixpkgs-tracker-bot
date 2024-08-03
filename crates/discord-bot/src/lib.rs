use std::sync::Arc;

use eyre::Result;
use log::trace;
use serenity::prelude::{Client, GatewayIntents, TypeMapKey};

mod config;
mod consts;
mod commands;
mod handler;
mod http;
mod jobs;
mod model;

use config::Config;
use handler::Handler;

/// Container for [`http::Client`]
struct SharedHttp;

impl TypeMapKey for SharedHttp {
	type Value = Arc<http::Client>;
}

/// Container for [`Config`]
struct SharedConfig;

impl TypeMapKey for SharedConfig {
	type Value = Arc<Config>;
}

/// Fetch our bot token
fn token() -> Result<String> {
	let token = std::env::var("DISCORD_BOT_TOKEN")?;
	Ok(token)
}

/// Create our client
///
/// # Errors
///
/// Will return [`Err`] if a [`Client`] cannot be created or configuration
/// cannot be created from the environment.
///
/// # Panics
///
/// Will [`panic!`] if the bot token isn't found or the ctrl+c handler can't be made
pub async fn client() -> Result<Client> {
	let token = token().expect("Couldn't find token in environment! Is DISCORD_BOT_TOKEN set?");

	let intents = GatewayIntents::default();
	trace!("Creating client");
	let client = Client::builder(token, intents)
		.event_handler(Handler)
		.await?;

	// add state stuff
	let http_client = <http::Client as http::ClientExt>::default();
	let config = Config::from_env()?;

	{
		let mut data = client.data.write().await;

		data.insert::<SharedHttp>(Arc::new(http_client));
		data.insert::<SharedConfig>(Arc::new(config.clone()));
	}

	let shard_manager = client.shard_manager.clone();

	// gracefully shutdown on ctrl+c
	tokio::spawn(async move {
		#[cfg(target_family = "unix")]
		tokio::signal::ctrl_c()
			.await
			.expect("Couldn't register ctrl+c handler!");
		shard_manager.shutdown_all().await;
	});

	// run our jobs
	jobs::dispatch(config)?;

	Ok(client)
}
