use bot_consts::{NIXPKGS_REMOTE, NIXPKGS_URL};
use bot_error::Error;

use std::{io::Write, path::Path};

use git2::{AutotagOption, FetchOptions, RemoteCallbacks, Repository};
use log::{debug, info, trace, warn};

pub const TTL_SECS: u64 = 60 * 5; // 5 minutes

// much of this is shamelessly lifted from
// https://github.com/rust-lang/git2-rs/blob/9a5c9706ff578c936be644dd1e8fe155bdc4d129/examples/pull.rs

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

/// update the given branches in the [`repository`] using the nixpkgs remote
fn update_branches_in(repository: &Repository, branches: &[String]) -> Result<(), Error> {
	let mut remote = repository.find_remote(NIXPKGS_REMOTE)?;
	// download all the refs
	remote.download(branches, Some(&mut fetch_options()))?;
	remote.disconnect()?;
	// and (hopefully) update what they refer to for later
	remote.update_tips(None, true, AutotagOption::Auto, None)?;

	Ok(())
}

pub fn fetch_or_update_repository(path: &str, branches: &[String]) -> Result<(), Error> {
	// Open our repository or clone it if it doesn't exist
	let path = Path::new(path);
	let repository = if path.exists() {
		Repository::open(path)?
	} else {
		warn!(
			"Couldn't find repository at {}! Cloning a fresh one from {NIXPKGS_URL}",
			path.display()
		);
		Repository::clone(NIXPKGS_URL, path)?;
		info!("Finished cloning to {}", path.display());

		// bail early as we already have a fresh copy
		return Ok(());
	};

	debug!("Updating repository at {}", path.display());
	update_branches_in(&repository, branches)?;
	debug!("Finished updating!");

	Ok(())
}
