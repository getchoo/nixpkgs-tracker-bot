use crate::{Error, RandomTeawie};

use std::future::Future;

use log::trace;

const TEAWIE_API: &str = "https://api.getchoo.com";

pub trait Ext {
	/// Get a random teawie
	///
	/// # Errors
	///
	/// Will return [`Err`] if the request fails or the response cannot be deserialized
	fn random_teawie(&self) -> impl Future<Output = Result<RandomTeawie, Error>> + Send;
}

impl Ext for super::Client {
	async fn random_teawie(&self) -> Result<RandomTeawie, Error> {
		let url = format!("{TEAWIE_API}/random_teawie");

		let request = self.get(&url).build()?;
		trace!("Making GET request to {}", request.url());
		let response = self.execute(request).await?;
		response.error_for_status_ref()?;
		let random_teawie: RandomTeawie = response.json().await?;

		Ok(random_teawie)
	}
}
