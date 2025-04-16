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
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
	auth::AuthSession,
	config::RessourcesRef,
	database::{
		ResolvedUserFeed,
		models::{self, Feed, NewUserFeed, UserFeedId},
	},
	import::{ImportedFeed, opml_to_feed_folders},
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
	feeds: Vec<ResolvedUserFeed<'a>>,
}

// Retrive feed entries
async fn feeds_get_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
) -> Result<Json<FeedsGetResponse<'static>>, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let mut conn = ressources.get_db_conn().unwrap();
	let user_feeds = ResolvedUserFeed::resolve_all(user_id, &mut conn).unwrap();

	Ok(Json(FeedsGetResponse { feeds: user_feeds }))
}

#[derive(Debug, Deserialize)]
struct FeedsPostRequest<'a> {
	title: Cow<'a, str>,
	description: Option<Cow<'a, str>>,
	url: Cow<'a, str>,
}

// Create new feed entries
async fn feeds_post_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
	Form(query): Form<FeedsPostRequest<'_>>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let FeedsPostRequest {
		title,
		description,
		url,
	} = query;

	let url = Url::parse(&url).unwrap();

	// TODO: assert url scheme is https (allow http?)

	let mut conn = ressources.get_db_conn().unwrap();

	let transaction = conn.transaction::<_, diesel::result::Error, _>(|conn| {
		let feed_id = Feed::resolve_or_create(&url, conn)?;

		let stmt = NewUserFeed {
			user_id,
			feed_id,
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
		Err((
			StatusCode::BAD_REQUEST,
			"the current user already has such a feed",
		)),
		|_id| {
			// TODO: return id?
			Ok(StatusCode::OK)
		},
	)
}

async fn feeds_delete_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
	Form(query): Form<FeedsPostRequest<'_>>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	todo!()
}

// Create new feed entries in bulk by using OPML format
async fn import_post_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
	mut multipart: Multipart,
) -> Result<StatusCode, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let mut folders = Vec::<(String, Vec<ImportedFeed>)>::new();
	while let Some(file) = multipart
		.next_field()
		.await
		.map_err(|err| (StatusCode::BAD_REQUEST, "could not read multipart field"))?
	{
		let bytes = file
			.bytes()
			.await
			.map_err(|err| (StatusCode::BAD_REQUEST, "could not decode file content"))?;

		let mut cursor = io::Cursor::new(bytes);
		let file_folders = opml_to_feed_folders(&mut cursor).unwrap();
		folders.extend(file_folders);
	}

	let mut conn = ressources.get_db_conn().unwrap();
	let transaction = conn.transaction::<(), diesel::result::Error, _>(move |conn| {
		for (folder_name, feeds) in folders {
			let resolved_values = feeds
				.into_iter()
				.map(|feed| {
					models::Feed::resolve_or_create(&feed.url, conn).map(|feed_id| NewUserFeed {
						user_id,
						feed_id,
						title: feed.title.into(),
						description: None,
					})
				})
				.collect::<QueryResult<Vec<_>>>()?;

			// TODO: do not crash on unique violation
			dsl::insert_into(crate::database::schema::user_feed::table)
				.values(resolved_values)
				.execute(conn)?;
		}
		Ok(())
	});

	transaction.unwrap();

	Ok(StatusCode::OK)
}
