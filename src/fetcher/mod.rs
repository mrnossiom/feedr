use bytes::Buf;
use diesel::{dsl, prelude::*};
use eyre::WrapErr;
use feed_rs::parser;
use reqwest::{Client, Response, StatusCode};
use tokio::{
	sync::mpsc::{self, Receiver},
	task,
};
use url::Url;

use crate::database::{PoolConnection, models::FeedId};

mod error;

pub use self::error::{Error, Result};

#[derive(Debug)]
pub struct Fetcher {
	client: Client,
	rx: Receiver<FetchTask>,
	db_pool: PoolConnection,
}

impl Fetcher {
	pub fn setup(db_pool: PoolConnection) -> eyre::Result<FetcherHandle> {
		// TODO: see how to handle large traffic
		let (tx, rx) = mpsc::channel(100);

		let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
		let client = Client::builder()
			.user_agent(user_agent)
			.build()
			.wrap_err("could not build client")?;

		let fetcher = Self {
			client,
			rx,
			db_pool,
		};
		fetcher.spawn();

		Ok(FetcherHandle { queue: tx })
	}

	fn spawn(self) {
		task::spawn(self.loop_task());
	}

	async fn loop_task(mut self) {
		while let Some(task) = self.rx.recv().await {
			if let Err(err) = self.task(task).await {
				tracing::error!(err = %err, "error while fetching");
			}
		}
	}

	async fn task(&self, task: FetchTask) -> Result<()> {
		let FetchTask { feed_id, url } = task;

		// TODO: log errors in the database to notify user

		// url.set_scheme("https")

		let response = self.client.get(url.clone()).send().await;
		let body = response
			.wrap_err("could not reach server")?
			.error_for_status()
			.wrap_err("server returned an error")?
			.bytes()
			.await
			.wrap_err("could not access request body")?;

		let parser = parser::Builder::new().sanitize_content(true).build();

		let feed = parser
			.parse(body.reader())
			.wrap_err("could not parse feed")?;

		tracing::debug!(feed_id = ?feed_id, url = %url, "sucessfully fetched feed");

		// let new_entries = feed.entries.iter().take_while(|feed| feed.updated)

		dbg!(feed);

		Ok(())
	}

	// async fn process(&self, task: FetchTask, mut conn: &mut PoolConnection) -> Result<(), FetchError> {}

	fn on_fetched(&self, feed_id: FeedId, url: &Url, response: &Response) -> Result<()> {
		use crate::database::schema::*;

		tracing::info!("sucessfully fetched url {url} for feed_id {feed_id:?}: {response:?}",);

		let mut conn = self.db_pool.get()?;
		conn.transaction::<_, eyre::Report, _>(|conn| {
			dsl::update(feed::table)
				.set(feed::status.eq("ok"))
				.execute(conn)
				.wrap_err("unable to update feed status")?;

			// TODO: filter and add feed entries

			Ok(())
		})?;

		Ok(())
	}

	fn on_failed(
		&self,
		feed_id: FeedId,
		url: &Url,
		err: &std::result::Result<StatusCode, reqwest::Error>,
	) -> Result<()> {
		use crate::database::schema::*;

		tracing::error!("unsucessfully fetched url {url} for feed_id {feed_id:?}: {err:?}");

		let mut conn = self.db_pool.get()?;
		dsl::update(feed::table)
			.set(feed::status.eq("failed"))
			.execute(&mut conn)
			.wrap_err("unable to update feed status")?;

		// TODO: add a custom error message in function of why it failed

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct FetcherHandle {
	queue: mpsc::Sender<FetchTask>,
}

impl FetcherHandle {
	pub async fn fetch_feed(&self, task: FetchTask) -> eyre::Result<()> {
		// TODO: this should not be blocking because of channel size
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
