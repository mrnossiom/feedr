use std::io;

use axum::{
	Json, Router,
	extract::{Multipart, State},
	http::StatusCode,
	routing::{get, post},
};
use diesel::{
	Connection, ExpressionMethods, Insertable, OptionalExtension, QueryDsl, RunQueryDsl,
	insert_into,
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
	entry_id: i32,

	title: String,
	description: String,
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

	use crate::models::schema::*;
	let mut conn = ressources.db_pool.get().unwrap();
	let user_id: Vec<(i32, String, String, String)> = feed_entry::table
		.inner_join(feed::table)
		.select((
			feed_entry::id,
			feed::url,
			feed_entry::title,
			feed_entry::description,
		))
		.filter(feed_entry::user_id.eq(user_id.0))
		.get_results(&mut conn)
		.unwrap();

	todo!()
}

// Create new feed entries
async fn feeds_post_handler(auth: AuthSession) -> Result<StatusCode, (StatusCode, &'static str)> {
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

	let (_folder_name, feeds) = folders[0];
	dbg!(&feeds);

	// TODO: resolve or insert feeds
	let mut conn = ressources.db_pool.get().unwrap();
	conn.transaction::<(), diesel::result::Error, _>(move |conn| {
		let feed_ids = Vec::new();

		for feed in feeds {
			use crate::models::schema::*;
			let id: Option<i32> = feed::table
				.select(feed::id)
				.filter(feed::url.eq(feed.url.as_str()))
				.get_result(conn)
				.optional()?;

			if let Some(id) = id {
				feed_ids.push(id);
			} else {
				insert_into(table).values(records)
			}
		}

		todo!();

		Ok(())
	})
	.unwrap();

	// TODO: register feeds and add them to user
	todo!();

	Ok(StatusCode::IM_A_TEAPOT)
}
