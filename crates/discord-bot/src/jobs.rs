use crate::{config::Config, consts::NIXPKGS_REMOTE, consts::NIXPKGS_URL};

use std::{path::Path, time::Duration};

use eyre::Result;
use git_tracker::ManagedRepository;
use log::error;

const TTL_SECS: u64 = 60 * 5; // 5 minutes

/// Run our jobs an initial time, then loop them on a separate thread
///
/// # Errors
///
/// Will return [`Err`] if any jobs fail
pub fn dispatch(config: Config) -> Result<()> {
	let managed_repository = ManagedRepository {
		path: Path::new(&config.nixpkgs_path).to_path_buf(),
		tracked_branches: config.nixpkgs_branches,
		upstream_remote_url: NIXPKGS_URL.to_string(),
		upstream_remote_name: NIXPKGS_REMOTE.to_string(),
	};

	managed_repository.fetch_or_update()?;

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(Duration::from_secs(TTL_SECS)).await;
			if let Err(why) = managed_repository.fetch_or_update() {
				error!("Could not fetch or update repository!\n{why:?}");
			};
		}
	});

	Ok(())
}
