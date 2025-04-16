use std::{borrow::Cow, io};

use axum::{
	Form, Json, Router,
	extract::{Multipart, State},
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
	config::Ressources,
	database::models::{Feed, NewUserFeed},
};
use crate::{
	database::models,
	import::{ImportedFeed, opml_to_feed_folders},
};

pub fn api_router() -> Router<Ressources> {
	Router::new()
		.route("/", get(feeds_get_handler).post(feeds_post_handler))
		.route("/import", post(import_post_handler))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FullUserFeed<'a> {
	id: i32,
	url: Cow<'a, str>,
	title: Cow<'a, str>,
	description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FeedsGetResponse<'a> {
	feeds: Vec<FullUserFeed<'a>>,
}

// Retrive feed entries
async fn feeds_get_handler(
	auth: AuthSession,
	State(ressources): State<Ressources>,
) -> Result<Json<FeedsGetResponse<'static>>, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let mut conn = ressources.get_db_conn().unwrap();

	use crate::database::schema::*;
	let user_feeds = user_feed::table
		.inner_join(feed::table)
		.select((
			user_feed::id,
			feed::url,
			user_feed::title,
			user_feed::description,
		))
		.filter(user_feed::user_id.eq(user_id))
		.load::<(i32, Cow<str>, Cow<str>, Option<Cow<str>>)>(&mut conn);

	let user_feeds = user_feeds.unwrap();

	let feeds = user_feeds
		.into_iter()
		.map(|(id, url, title, description)| FullUserFeed {
			id,
			url,
			title,
			description,
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

		let id = match stmt.get_result::<i32>(conn) {
			Ok(id) => Some(id),
			Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => None,
			_ => todo!(),
		};

		Ok(id)
	});

	let id = transaction.unwrap();

	id.map_or(
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
	let mut conn = ressources.get_db_conn().unwrap();
	let transaction = conn.transaction::<(), diesel::result::Error, _>(move |conn| {
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

		dsl::insert_into(crate::database::schema::user_feed::table)
			.values(resolved_values)
			.execute(conn)?;

		Ok(())
	});

	transaction.unwrap();

	Ok(StatusCode::OK)
}
