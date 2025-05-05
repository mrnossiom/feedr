use axum::{Json, Router, routing::get};
use eyre::Context;
use serde::Serialize;

use crate::{
	config::RessourcesRef,
	database::ResolvedUserEntry,
	front::{auth::ApiSession, error::RouteResult},
};

pub fn router() -> Router<RessourcesRef> {
	Router::new().route(
		"/",
		get(entries_get_handler), // .post(entries_post_handler)
		                          // .delete(entries_post_handler),
	)
}

#[derive(Debug, Clone, Serialize)]
struct EntriesGetResponse<'a> {
	user_feed_entries: Vec<ResolvedUserEntry<'a>>,
}

// Retrive feed entries
async fn entries_get_handler<'a>(
	auth: ApiSession,
	ressources: RessourcesRef,
) -> RouteResult<Json<EntriesGetResponse<'a>>> {
	let user_id = auth.user_id()?;

	let mut conn = ressources.database_handle.get()?;
	let user_feed_entries = ResolvedUserEntry::resolve_all(user_id, &mut conn)
		.wrap_err("could not retrieve user feed entries")?;

	Ok(Json(EntriesGetResponse { user_feed_entries }))
}
