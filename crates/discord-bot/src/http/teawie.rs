use crate::model::RandomTeawie;

use std::future::Future;

use eyre::Result;
use log::trace;

const TEAWIE_API: &str = "https://api.getchoo.com";

pub trait Ext {
	/// Get a random teawie
	///
	/// # Errors
	///
	/// Will return [`Err`] if the request fails or the response cannot be deserialized
	fn random_teawie(&self) -> impl Future<Output = Result<RandomTeawie>> + Send;
}

impl Ext for super::Client {
	async fn random_teawie(&self) -> Result<RandomTeawie> {
		let url = format!("{TEAWIE_API}/random_teawie");

		trace!("Making GET request to {url}");
		let request = self.get(&url);
		let response = request.send().await?;
		response.error_for_status_ref()?;
		let random_teawie: RandomTeawie = response.json().await?;

		Ok(random_teawie)
	}
}
