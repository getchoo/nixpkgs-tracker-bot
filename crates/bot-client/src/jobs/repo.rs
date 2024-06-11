use crate::{Error, NIXPKGS_BRANCHES, NIXPKGS_REMOTE, NIXPKGS_URL};

use std::{io::Write, path::Path};

use git2::{FetchOptions, Remote, RemoteCallbacks, Repository};
use log::{debug, trace, warn};

pub const TTL_SECS: u64 = 60 * 5; // 5 minutes

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

fn download_remote<'a>(repository: &'a Repository, remote_name: &str) -> Result<Remote<'a>, Error> {
	trace!("Downloading remote {remote_name}");
	let mut remote = repository.find_remote(remote_name)?;
	remote.download(&[] as &[&str], Some(&mut fetch_options()))?;
	remote.disconnect()?;

	Ok(remote)
}

fn fetch_refs(remote: &mut Remote, refs: &[&str]) -> Result<(), Error> {
	trace!("Fetching refs: {refs:?}");
	Ok(remote.fetch(refs, Some(&mut fetch_options()), None)?)
}

pub fn fetch_or_update_repository(path: &str) -> Result<(), Error> {
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

		// bail early as we already have a fresh copy
		return Ok(());
	};

	debug!("Updating repository at {}", path.display());
	let mut remote = download_remote(&repository, NIXPKGS_REMOTE)?;
	fetch_refs(&mut remote, &NIXPKGS_BRANCHES)?;
	debug!("Finished updating!");

	Ok(())
}
