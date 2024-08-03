use crate::config::Config;

use std::time::Duration;

use eyre::Result;
use log::error;

mod repo;

/// Run our jobs an initial time, then loop them on a separate thread
///
/// # Errors
///
/// Will return [`Err`] if any jobs fail
pub fn dispatch(config: Config) -> Result<()> {
	repo::fetch_or_update_repository(&config.nixpkgs_path, &config.nixpkgs_branches)?;

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(Duration::from_secs(repo::TTL_SECS)).await;
			if let Err(why) =
				repo::fetch_or_update_repository(&config.nixpkgs_path, &config.nixpkgs_branches)
			{
				error!("Failed to fetch or update repository!\n{why:?}");
			};
		}
	});

	Ok(())
}
