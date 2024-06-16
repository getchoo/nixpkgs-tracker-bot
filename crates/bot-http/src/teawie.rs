use super::{ClientExt as _, Error};
use crate::model::RandomTeawie;

use std::future::Future;

const TEAWIE_API: &str = "https://api.getchoo.com";

pub trait ClientExt {
	/// Get a random teawie
	///
	/// # Errors
	///
	/// Will return [`Err`] if the request fails or the response cannot be deserialized
	fn random_teawie(&self) -> impl Future<Output = Result<Option<String>, Error>> + Send;
}

impl ClientExt for super::Client {
	async fn random_teawie(&self) -> Result<Option<String>, Error> {
		let url = format!("{TEAWIE_API}/random_teawie");
		let resp: RandomTeawie = self.get_json(&url).await?;

		Ok(resp.url)
	}
}
