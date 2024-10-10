//! Library for helping you track commits and branches in a Git repository
use std::path::PathBuf;

use git2::{
	BranchType, FetchOptions, FetchPrune, Oid, Reference, RemoteCallbacks, RemoteUpdateFlags,
	Repository,
};
use log::{debug, info, trace};

/// Used when logging Git transfer progress
const INCREMENT_TO_LOG: i32 = 5;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("libgit2 error")]
	Git(#[from] git2::Error),
	#[error("i/o error")]
	IOError(#[from] std::io::Error),
}

/// Helper struct for tracking Git objects
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct TrackedRepository {
	/// Path to repository
	path: PathBuf,
	/// URL of the Git remote
	remote_url: String,
	/// Name of the remote referring to `remote_url`
	remote_name: String,
}

impl TrackedRepository {
	#[must_use]
	pub fn new(path: PathBuf, remote_url: String, remote_name: String) -> Self {
		Self {
			path,
			remote_url,
			remote_name,
		}
	}

	/// Open a [`Repository`]
	///
	/// # Errors
	///
	/// Will return [`Err`] if the repository cannot be opened
	pub fn open(&self) -> Result<Repository, Error> {
		trace!("Opening repository at {}", self.path.display());
		Ok(Repository::open(&self.path)?)
	}

	/// Clone a (small) fresh copy of your repository
	///
	/// # Errors
	///
	/// Will return [`Err`] if the path, repository, or remote cannot be created
	pub fn clone_repository(&self) -> Result<(), Error> {
		// Setup a bare repository to save space
		info!("Creating repository at {}", self.path.display());
		std::fs::create_dir_all(&self.path)?;
		let repository = Repository::init_bare(&self.path)?;

		debug!("Adding remote {} for {}", self.remote_name, self.remote_url,);
		repository.remote(&self.remote_name, &self.remote_url)?;
		self.fetch()?;

		Ok(())
	}

	#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
	fn fetch_options<'a>() -> FetchOptions<'a> {
		let mut rc = RemoteCallbacks::new();

		// Log transfer progress
		let mut current_percentage = 1;
		rc.transfer_progress(move |stats| {
			if stats.received_objects() == stats.total_objects() {
				// HACK: Avoid dividing by zero
				// I have no idea how this can ever be zero but ok
				let total_deltas = stats.total_deltas();
				if total_deltas == 0 {
					return true;
				}

				let percentage =
					(stats.indexed_deltas() as f32 / stats.total_deltas() as f32 * 100.0) as i32;
				if percentage != current_percentage && percentage % INCREMENT_TO_LOG == 0 {
					info!(
						"Resolving deltas {}/{}\r",
						stats.indexed_deltas(),
						stats.total_deltas()
					);
					current_percentage = percentage;
				}
			} else if stats.total_objects() > 0 {
				let percentage =
					(stats.received_objects() as f32 / stats.total_objects() as f32 * 100.0) as i32;
				if percentage != current_percentage && percentage % INCREMENT_TO_LOG == 0 {
					info!(
						"Received {}/{} objects ({}) in {} bytes\r",
						stats.received_objects(),
						stats.total_objects(),
						stats.indexed_objects(),
						stats.received_bytes()
					);
					current_percentage = percentage;
				}
			}

			true
		});

		// Log ref updates
		rc.update_tips(|refname, orig_oid, new_oid| {
			if orig_oid.is_zero() {
				info!("[new]   {:20} {}", new_oid, refname);
			} else {
				info!("[updated]   {:10}..{:10} {}", orig_oid, new_oid, refname);
			}
			true
		});

		let mut fetch_options = FetchOptions::new();
		// Make sure we prune on fetch
		fetch_options.prune(FetchPrune::On).remote_callbacks(rc);

		fetch_options
	}

	/// Fetch the tracked remote
	///
	/// # Errors
	///
	/// Will return [`Err`] if the repository cannot be opened, the remote cannot be found, the
	/// refs cannot be fetched, or the tips of the refs cannot be updated
	pub fn fetch(&self) -> Result<(), Error> {
		let repository = self.open()?;

		let mut remote = repository.find_remote(&self.remote_name)?;

		info!("Fetching repository");
		remote.download(&[] as &[&str], Some(&mut Self::fetch_options()))?;
		remote.disconnect()?;

		debug!("Updating tips");
		remote.update_tips(
			None,
			RemoteUpdateFlags::UPDATE_FETCHHEAD,
			git2::AutotagOption::None,
			None,
		)?;

		Ok(())
	}

	/// Check if a [`Reference`] contains a given Git object
	///
	/// # Errors
	///
	/// Will return [`Err`] if the repository cannot be opened, HEAD cannot be resolved, or the
	/// relation between commits cannot be resolved
	pub fn ref_contains_object(&self, reference: &Reference, commit: Oid) -> Result<bool, Error> {
		trace!(
			"Checking for commit {commit} in {}",
			reference.name().unwrap_or("<branch>")
		);
		let repository = self.open()?;
		let head = reference.peel_to_commit()?;

		// NOTE: we have to check this as `Repository::graph_descendant_of()` (like the name says)
		// only finds *descendants* of it's parent commit, and will not tell us if the parent commit
		// *is* the child commit. i have no idea why i didn't think of this, but that's why this
		// comment is here now
		if head.id() == commit {
			return Ok(true);
		}

		let has_commit = repository.graph_descendant_of(head.id(), commit)?;

		Ok(has_commit)
	}

	/// Check if multiple [`Reference`]s contain a commit SHA
	///
	/// # Errors
	///
	/// Will return [`Err`] if an [`Oid`] could not be resolved from the commit SHA
	/// or when it can't be determined if a reference contains a commit
	pub fn branches_contain_sha<'a>(
		&self,
		branch_names: impl IntoIterator<Item = &'a String>,
		commit_sha: &str,
	) -> Result<Vec<(&'a String, bool)>, Error> {
		let repository = self.open()?;
		let commit = Oid::from_str(commit_sha)?;

		let mut results = vec![];
		for branch_name in branch_names {
			let branch = repository.find_branch(
				&format!("{}/{branch_name}", self.remote_name),
				BranchType::Remote,
			)?;

			let has_commit = self.ref_contains_object(&branch.into_reference(), commit)?;
			results.push((branch_name, has_commit));
		}

		Ok(results)
	}
}
