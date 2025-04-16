use eyre::WrapErr;
use reqwest::Client;
use tokio::{
	sync::mpsc::{self, Receiver},
	task,
};
use url::Url;

use crate::database::models::FeedId;

#[derive(Debug)]
pub struct Fetcher {
	client: Client,
	rx: Receiver<FetchTask>,
}

impl Fetcher {
	pub fn setup() -> eyre::Result<FetcherHandle> {
		// TODO: see how to handle large traffic
		let (tx, rx) = mpsc::channel(100);

		let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
		let client = Client::builder()
			.user_agent(user_agent)
			.build()
			.wrap_err("could not build client")?;

		let fetcher = Self { client, rx };
		fetcher.spawn();

		Ok(FetcherHandle { queue: tx })
	}

	fn spawn(self) {
		task::spawn(self.loop_task());
	}

	async fn loop_task(mut self) {
		while let Some(task) = self.rx.recv().await {
			let FetchTask { feed_id, url } = task;

			match self.client.get(url.clone()).send().await {
				Ok(res) => {
					tracing::info!(
						"sucessfully fetched url {url} for feed_id {feed_id:?}: {res:?}",
					);
					// todo!();
				}
				Err(err) => {
					tracing::error!(
						"unsucessfully fetched url {url} for feed_id {feed_id:?}: {err:?}"
					);
				}
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct FetcherHandle {
	queue: mpsc::Sender<FetchTask>,
}

impl FetcherHandle {
	pub async fn fetch_feed(&self, task: FetchTask) -> eyre::Result<()> {
		// TODO: this should not be blocking
		self.queue
			.send(task)
			.await
			.wrap_err("could not send fetch task to executor")
	}
}

pub struct FetchTask {
	pub feed_id: FeedId,
	pub url: Url,
}
