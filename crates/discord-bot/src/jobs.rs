use crate::config::Config;

use std::time::Duration;

use eyre::Result;
use log::error;

const TTL_SECS: u64 = 60 * 5; // 5 minutes

/// Run our jobs an initial time, then loop them on a separate thread
///
/// # Errors
///
/// Will return [`Err`] if any jobs fail
pub fn dispatch(config: &Config) -> Result<()> {
	let repository = config.repository();
	if repository.open().is_err() {
		repository.clone_repository()?;
	}
	repository.fetch()?;

	let repository_clone = repository.clone();

	tokio::spawn(async move {
		loop {
			tokio::time::sleep(Duration::from_secs(TTL_SECS)).await;
			if let Err(why) = repository_clone.fetch() {
				error!("Could not fetch or update repository!\n{why:?}");
			};
		}
	});

	Ok(())
}
