use eyre::WrapErr;
use reqwest::Client;
use tokio::{sync::mpsc, task};
use url::Url;

#[derive(Debug)]
pub struct Fetcher {
	client: Client,
}

impl Fetcher {
	pub fn setup() -> eyre::Result<FetcherHandle> {
		let (tx, mut rx) = mpsc::channel(100);

		let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
		let client = Client::builder()
			.user_agent(user_agent)
			.build()
			.wrap_err("could not build client")?;
		let fetcher = Self { client };

		task::spawn(async move {
			while let Some(task) = rx.recv().await {
				let FetchTask { feed_id, url } = task;

				match fetcher.client.get(url.clone()).send().await {
					Ok(res) => {
						tracing::info!(
							"sucessfully fetched url {url} for feed_id {feed_id}: {res:?}",
						);
						// todo!();
					}
					Err(err) => {
						tracing::error!(
							"unsucessfully fetched url {url} for feed_id {feed_id}: {err:?}"
						);
					}
				}
			}
		});

		Ok(FetcherHandle { queue: tx })
	}
}

#[derive(Debug, Clone)]
pub struct FetcherHandle {
	queue: mpsc::Sender<FetchTask>,
}

impl FetcherHandle {
	async fn fetch_feed(&self, task: FetchTask) -> eyre::Result<()> {
		self.queue
			.send(task)
			.await
			.wrap_err("could not send fetch task to executor")
	}
}

struct FetchTask {
	feed_id: i32,
	url: Url,
}
