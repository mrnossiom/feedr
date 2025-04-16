use std::{borrow::Cow, io};

use axum::{
	Json, Router,
	extract::{Multipart, Query, State},
	http::StatusCode,
	routing::{get, post},
};
use diesel::{
	Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, insert_into,
};
use serde::{Deserialize, Serialize};

use crate::import::{ImportedFeed, opml_to_feed_folders};
use crate::{
	auth::{AuthSession, AuthnLayer},
	config::Ressources,
};

pub fn api_router(ressources: Ressources) -> Router<Ressources> {
	let authn_layer = AuthnLayer::new_with_api_keys(ressources.db_pool);

	Router::new()
		.nest("/v0", nightly_api_router())
		.layer(authn_layer)
}

pub fn nightly_api_router() -> Router<Ressources> {
	Router::new().nest("/feeds", feeds_api_router())
}

pub fn feeds_api_router() -> Router<Ressources> {
	Router::new()
		.route("/", get(feeds_get_handler).post(feeds_post_handler))
		.route("/import", post(import_post_handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Feed {
	id: i32,
	title: String,
	description: Option<String>,
	url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedsGetResponse {
	feeds: Vec<Feed>,
}

// Retrive feed entries
async fn feeds_get_handler(
	auth: AuthSession,
	State(ressources): State<Ressources>,
) -> Result<Json<FeedsGetResponse>, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	use crate::database::schema::*;
	let mut conn = ressources.db_pool.get().unwrap();
	let user_feeds = user_feed::table
		.inner_join(feed::table)
		.select((
			user_feed::id,
			feed::url,
			user_feed::title,
			user_feed::description,
		))
		.filter(user_feed::user_id.eq(user_id.0))
		.load::<(i32, String, String, Option<String>)>(&mut conn)
		.unwrap();

	let feeds = user_feeds
		.into_iter()
		.map(|(id, url, title, description)| Feed {
			id,
			title,
			description,
			url,
		})
		.collect::<Vec<_>>();

	Ok(Json(FeedsGetResponse { feeds }))
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
	State(ressources): State<Ressources>,
	Query(query): Query<FeedsPostRequest<'_>>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let FeedsPostRequest {
		title,
		description,
		url,
	} = query;

	todo!()
}

// Create new feed entries in bulk by using OPML format
async fn import_post_handler(
	auth: AuthSession,
	State(ressources): State<Ressources>,
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

	let (_folder_name, feeds) = folders.swap_remove(0);

	// TODO: resolve or insert feeds
	let mut conn = ressources.db_pool.get().unwrap();
	conn.transaction::<(), diesel::result::Error, _>(move |conn| {
		use crate::database::schema::*;

		let mut resolved_feeds = Vec::new();

		for feed in feeds {
			let id = feed::table
				.select(feed::id)
				.filter(feed::url.eq(feed.url.as_str()))
				.get_result::<i32>(conn)
				.optional()?;

			if let Some(id) = id {
				resolved_feeds.push((feed.title, id));
			} else {
				let feed_id = insert_into(feed::table)
					.values((feed::url.eq(feed.url.as_str()), feed::status.eq(1)))
					.returning(feed::id)
					.get_result::<i32>(conn)?;
				resolved_feeds.push((feed.title, feed_id));
			}
		}

		let values = resolved_feeds
			.into_iter()
			.map(|(title, id)| {
				(
					user_feed::feed_id.eq(id),
					user_feed::user_id.eq(user_id.0),
					user_feed::title.eq(title),
				)
			})
			.collect::<Vec<_>>();

		insert_into(user_feed::table).values(values).execute(conn)?;

		Ok(())
	})
	.unwrap();

	Ok(StatusCode::OK)
}
