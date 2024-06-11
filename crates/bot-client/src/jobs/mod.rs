use crate::{Config, Error};

use std::time::Duration;

use log::error;

mod repo;

/// Run our jobs an initial time, then loop them on a separate thread
pub fn dispatch(config: Config) -> Result<(), Error> {
	repo::fetch_or_update_repository(&config.nixpkgs_path)?;
	tokio::spawn(async move {
		loop {
			tokio::time::sleep(Duration::from_secs(repo::TTL_SECS)).await;
			if let Err(why) = repo::fetch_or_update_repository(&config.nixpkgs_path) {
				error!("Failed to fetch or update repository!\n{why:?}");
			};
		}
	});

	Ok(())
}
