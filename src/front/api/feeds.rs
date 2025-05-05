use std::{borrow::Cow, io};

use axum::{
	Form, Json, Router,
	extract::Multipart,
	http::StatusCode,
	routing::{get, post},
};
use diesel::{
	dsl,
	prelude::*,
	result::{DatabaseErrorKind, Error},
};
use eyre::Context;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
	config::RessourcesRef,
	database::{
		ResolvedUserFeed,
		models::{self, Feed, NewUserFeed, UserFeedFolder, UserFeedId},
	},
	front::{
		auth::ApiSession,
		error::{RouteError, RouteResult},
	},
	utils::{ImportedFeed, opml_to_feed_folders},
};

pub fn router() -> Router<RessourcesRef> {
	Router::new()
		.route(
			"/",
			get(feeds_get_handler)
				.post(feeds_post_handler)
				.delete(feeds_delete_handler),
		)
		.route("/import", post(import_post_handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedsGetResponse<'a> {
	user_feeds: Vec<ResolvedUserFeed<'a>>,
}

// Retrive feed entries
async fn feeds_get_handler(
	auth: ApiSession,
	ressources: RessourcesRef,
) -> RouteResult<Json<FeedsGetResponse<'static>>> {
	let user_id = auth.user_id()?;

	let mut conn = ressources.database_handle.get()?;
	let user_feeds = ResolvedUserFeed::resolve_all(user_id, &mut conn)
		.wrap_err("could not retrieve user feeds")?;

	Ok(Json(FeedsGetResponse { user_feeds }))
}

#[derive(Debug, Deserialize)]
struct FeedsPostRequest<'a> {
	title: Cow<'a, str>,
	description: Option<Cow<'a, str>>,
	url: Cow<'a, str>,
}

// Create new feed entries
async fn feeds_post_handler(
	auth: ApiSession,
	ressources: RessourcesRef,
	Form(query): Form<FeedsPostRequest<'_>>,
) -> RouteResult<StatusCode> {
	let user_id = auth.user_id()?;

	let FeedsPostRequest {
		title,
		description,
		url,
	} = query;

	let url = Url::parse(&url).map_err(|_| RouteError::User("url is not valid"))?;

	// TODO: assert url scheme is https (allow http?)

	let mut conn = ressources.database_handle.get()?;

	let transaction = conn.transaction::<_, diesel::result::Error, _>(|conn| {
		let feed_id = Feed::resolve_or_create(&url, conn)?;

		let stmt = NewUserFeed {
			user_id,
			feed_id,
			folder_id: None,
			title,
			description,
		}
		.insert_into(crate::database::schema::user_feed::table)
		.returning(crate::database::schema::user_feed::id);

		let user_feed_id = match stmt.get_result::<UserFeedId>(conn) {
			Ok(id) => Some(id),
			Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => None,
			Err(err) => return Err(err),
		};

		Ok((feed_id, user_feed_id))
	});

	let (feed_id, user_feed_id) = transaction.unwrap();

	ressources.fetch_url(feed_id, url).await.unwrap();

	user_feed_id.map_or(
		Err(RouteError::User("the current user already has such a feed")),
		|_id| {
			// TODO: return id?
			Ok(StatusCode::OK)
		},
	)
}

#[derive(Debug, Deserialize)]
struct FeedsDeleteRequest {
	id: UserFeedId,
}

async fn feeds_delete_handler(
	auth: ApiSession,
	ressources: RessourcesRef,
	Form(query): Form<FeedsDeleteRequest>,
) -> RouteResult<StatusCode> {
	let user_id = auth.user_id()?;

	todo!()
}

// Create new feed entries in bulk by using OPML format
async fn import_post_handler(
	auth: ApiSession,
	ressources: RessourcesRef,
	mut multipart: Multipart,
) -> RouteResult<StatusCode> {
	let user_id = auth.user_id()?;

	let mut folders = Vec::<(String, Vec<ImportedFeed>)>::new();
	while let Some(file) = multipart
		.next_field()
		.await
		.map_err(|err| RouteError::UserOpaque("could not read multipart field", err.into()))?
	{
		let bytes = file
			.bytes()
			.await
			.map_err(|err| RouteError::UserOpaque("could not decode file content", err.into()))?;

		let mut cursor = io::Cursor::new(bytes);
		let file_folders = opml_to_feed_folders(&mut cursor)
			.wrap_err("could not transform opml file in list of folders")?;
		folders.extend(file_folders);
	}

	let mut to_fetch = Vec::new();

	let mut conn = ressources.database_handle.get()?;
	conn.transaction::<(), diesel::result::Error, _>(|conn| {
		for (folder_name, feeds) in folders {
			let folder_id = UserFeedFolder::resolve_or_create(user_id, &folder_name, conn)?;

			for feed in feeds {
				let new_feed = models::Feed::resolve_or_create(&feed.url, conn).map(|feed_id| {
					NewUserFeed {
						user_id,
						feed_id,
						folder_id: Some(folder_id),
						title: feed.title.into(),
						description: None,
					}
				})?;

				to_fetch.push((new_feed.feed_id, feed.url));

				// TODO: do not crash on unique violation
				dsl::insert_into(crate::database::schema::user_feed::table)
					.values(new_feed)
					.execute(conn)?;
			}
		}
		Ok(())
	})
	.wrap_err("failed to register bulk feeds from opml file")?;

	for (feed_id, url) in to_fetch {
		ressources
			.fetch_url(feed_id, url)
			.await
			.wrap_err("failed to put feed in fetcher queue")?;
	}

	Ok(StatusCode::OK)
}
