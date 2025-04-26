use std::{borrow::Cow, fmt::Write};

use askama::Template;
use axum::{
	Form, Router,
	extract::Query,
	response::{Html, IntoResponse, Redirect, Response},
	routing::get,
};
use axum_login::login_required;
use eyre::Context;
use serde::Deserialize;
use templates::{IndexTemplate, LoginTemplate, ProfileTemplate};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
	auth::{AuthSession, Backend, LoginCredentials},
	config::RessourcesRef,
	database::{ResolvedUserEntry, ResolvedUserFeed},
	error::RouteResult,
};

mod templates;

pub fn router() -> Router<RessourcesRef> {
	let protected = Router::new()
		// TODO: show an about page to newcomers
		.route("/", get(root_get_handler))
		.route("/profile", get(profile_get_handler))
		.nest("/web", web_fragment_router())
		.route_layer(login_required!(Backend, login_url = "/login"));

	protected
		.route("/profile/logout", get(profile_logout_get_handler))
		.route("/login", get(login_get_handler).post(login_post_handler))
		.route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
		.nest_service("/static", ServeDir::new("static"))
}

pub fn web_fragment_router() -> Router<RessourcesRef> {
	Router::new().route("/", get(async || "Hello, FeedR!"))
}

async fn root_get_handler(auth: AuthSession, ressources: RessourcesRef) -> RouteResult<Response> {
	let user = auth.user.unwrap();

	let mut conn = ressources.database_handle.get()?;
	let user_feeds = ResolvedUserFeed::resolve_all_by_folders(user.id, &mut conn)
		.wrap_err("could not retrieve user feeds")?;
	let user_entries = ResolvedUserEntry::resolve_all(user.id, &mut conn)
		.wrap_err("could not retrieve entries")?;

	let template = IndexTemplate {
		user: &user,
		user_feeds,
		user_entries,
	};

	Ok(Html(template.render()?).into_response())
}

async fn profile_get_handler(auth: AuthSession) -> RouteResult<Response> {
	let user = auth.user.unwrap();

	let tpl = ProfileTemplate { user: &user };
	Ok(Html(tpl.render()?).into_response())
}

async fn profile_logout_get_handler(mut auth: AuthSession) -> RouteResult<Redirect> {
	auth.logout().await?;
	Ok(Redirect::to("/"))
}

async fn login_get_handler(auth: AuthSession) -> RouteResult<Response> {
	let template = LoginTemplate {};

	Ok(Html(template.render()?).into_response())
}

#[derive(Debug, Deserialize)]
struct LoginNext<'a> {
	next: Option<Cow<'a, str>>,
}

async fn login_post_handler(
	mut auth: AuthSession,
	Query(LoginNext { next }): Query<LoginNext<'_>>,
	Form(login): Form<LoginCredentials>,
) -> RouteResult<Redirect> {
	let Some(user) = auth.authenticate(login).await? else {
		let mut login_url = "/login".to_string();
		if let Some(next) = next {
			let _ = write!(login_url, "?next={next}");
		}
		return Ok(Redirect::to(&login_url));
	};

	auth.login(&user).await?;

	next.map_or_else(|| Ok(Redirect::to("/")), |next| Ok(Redirect::to(&next)))
}
