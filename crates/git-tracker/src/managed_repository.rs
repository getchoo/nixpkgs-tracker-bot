use git2::{AutotagOption, FetchOptions, RemoteCallbacks, RemoteUpdateFlags, Repository};
use log::{debug, info, trace, warn};
use std::{io::Write, path::PathBuf};

// much of this is shamelessly lifted from
// https://github.com/rust-lang/git2-rs/blob/9a5c9706ff578c936be644dd1e8fe155bdc4d129/examples/pull.rs

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("libgit2 error")]
	Git(#[from] git2::Error),
}

pub struct ManagedRepository {
	pub path: PathBuf,
	pub tracked_branches: Vec<String>,
	pub upstream_remote_url: String,
	pub upstream_remote_name: String,
}

impl ManagedRepository {
	/// basic set of options for fetching from remotes
	fn fetch_options<'a>() -> FetchOptions<'a> {
		let mut remote_callbacks = RemoteCallbacks::new();
		remote_callbacks.transfer_progress(|progress| {
			if progress.received_objects() == progress.total_objects() {
				trace!(
					"Resolving deltas {}/{}\r",
					progress.indexed_deltas(),
					progress.total_deltas()
				);
			} else {
				trace!(
					"Received {}/{} objects ({}) in {} bytes\r",
					progress.received_objects(),
					progress.total_objects(),
					progress.indexed_objects(),
					progress.received_bytes()
				);
			}
			std::io::stdout().flush().ok();
			true
		});

		let mut fetch_opts = FetchOptions::new();
		fetch_opts.remote_callbacks(remote_callbacks);

		fetch_opts
	}

	/// Update the given branches in the [`repository`] using the nixpkgs remote
	fn update_branches_in(&self, repository: &Repository) -> Result<(), Error> {
		let mut remote = repository.find_remote(&self.upstream_remote_url)?;
		// download all the refs
		remote.download(&self.tracked_branches, Some(&mut Self::fetch_options()))?;
		remote.disconnect()?;
		// and (hopefully) update what they refer to for later
		remote.update_tips(None, RemoteUpdateFlags::UPDATE_FETCHHEAD, AutotagOption::Auto, None)?;

		Ok(())
	}

	/// Fetch the repository or update it if it exists
	///
	/// # Errors
	/// Will return [`Err`] if the repository cannot be opened, cloned, or updated
	pub fn fetch_or_update(&self) -> Result<(), Error> {
		// Open our repository or clone it if it doesn't exist
		let repository = if self.path.exists() {
			Repository::open(self.path.as_path())?
		} else {
			warn!(
				"Couldn't find repository at {}! Cloning a fresh one from {}",
				self.path.display(),
				self.upstream_remote_url
			);
			Repository::clone(&self.upstream_remote_url, self.path.as_path())?;
			info!("Finished cloning to {}", self.path.display());

			// bail early as we already have a fresh copy
			return Ok(());
		};

		debug!("Updating repository at {}", self.path.display());
		self.update_branches_in(&repository)?;
		debug!("Finished updating!");

		Ok(())
	}
}
