use axum::{Json, Router, http::StatusCode, routing::get};
use serde::Serialize;

use crate::{auth::AuthSession, config::RessourcesRef, database::ResolvedUserEntry};

pub fn router() -> Router<RessourcesRef> {
	Router::new().route(
		"/",
		get(entries_get_handler), // .post(feeds_post_handler)
		                          // .delete(feeds_post_handler),
	)
}

#[derive(Debug, Clone, Serialize)]
struct EntriesGetResponse<'a> {
	user_feed_entries: Vec<ResolvedUserEntry<'a>>,
}

// Retrive feed entries
async fn entries_get_handler<'a>(
	auth: AuthSession,
	ressources: RessourcesRef,
) -> Result<Json<EntriesGetResponse<'a>>, (StatusCode, &'static str)> {
	let Some(user_id) = auth.user_id else {
		return Err((StatusCode::UNAUTHORIZED, "you are not logged in"));
	};

	let mut conn = ressources.get_db_conn().unwrap();
	let user_feed_entries = ResolvedUserEntry::resolve_all(user_id, &mut conn).unwrap();

	Ok(Json(EntriesGetResponse { user_feed_entries }))
}
