use std::borrow::Cow;

use axum::{
	Form, Router,
	extract::Query,
	response::{IntoResponse, Redirect, Response},
	routing::get,
};
use axum_login::login_required;
use eyre::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use time::Duration;
use tower_cookies::{Cookie, Cookies};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
	config::RessourcesRef,
	database::{ResolvedUserEntry, ResolvedUserFeed},
	front::{
		auth::{AuthSession, Backend, LoginCredentials, UserSession, is_safe_relative_path},
		error::RouteResult,
		web::{
			htmx::{HxNotices, HxRedirect, HxResponse, IntoHxResponse},
			templates::Template,
		},
	},
};

mod htmx;
mod templates;

pub const LOGIN_NEXT_COOKIE_NAME: &str = "login-next";

pub fn router() -> Router<RessourcesRef> {
	let protected = Router::new()
		// TODO: show an about page to newcomers
		.route("/", get(root_get_handler))
		.route("/profile", get(profile_get_handler))
		.nest("/web", web_fragment_router())
		.route_layer(login_required!(Backend, login_url = "/login"));

	protected
		// public routes
		.route("/logout", get(profile_logout_get_handler))
		.route("/login", get(login_get_handler).post(login_post_handler))
		.route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
		.nest_service("/static", ServeDir::new("static"))
		.fallback(not_found)
}

pub fn web_fragment_router() -> Router<RessourcesRef> {
	Router::new().route("/", get(async || "Hello, FeedR!"))
}

async fn not_found(auth: AuthSession) -> impl IntoResponse {
	let template = templates::NotFound {
		user: auth.user.as_ref(),
	};
	(StatusCode::NOT_FOUND, Template::render(&template))
}

async fn root_get_handler(
	UserSession(user): UserSession,
	ressources: RessourcesRef,
) -> RouteResult<Template> {
	let mut conn = ressources.database_handle.get()?;
	let user_feeds = ResolvedUserFeed::resolve_all_by_folders(user.id, &mut conn)
		.wrap_err("could not retrieve user feeds")?;
	let user_entries = ResolvedUserEntry::resolve_all(user.id, &mut conn)
		.wrap_err("could not retrieve entries")?;

	let tpl = templates::Index {
		user: Some(&user),
		user_feeds,
		user_entries,
	};
	Ok(Template::render(&tpl))
}

async fn profile_get_handler(UserSession(user): UserSession) -> RouteResult<Template> {
	let tpl = templates::Profile { user: Some(&user) };
	Ok(Template::render(&tpl))
}

async fn profile_logout_get_handler(mut auth: AuthSession) -> RouteResult<Redirect> {
	auth.logout().await?;
	Ok(Redirect::to("/"))
}

#[derive(Debug, Deserialize)]
struct LoginNext<'a> {
	next: Option<Cow<'a, str>>,
}

async fn login_get_handler(
	auth: AuthSession,
	cookies: Cookies,
	Query(LoginNext { next }): Query<LoginNext<'_>>,
) -> RouteResult<Response> {
	if auth.user.is_some() {
		return Ok(Redirect::to(next.as_deref().unwrap_or("/")).into_response());
	}

	if let Some(next) = next
		&& next != "/"
	{
		let mut cookie = Cookie::new(LOGIN_NEXT_COOKIE_NAME, next.into_owned());
		cookie.set_max_age(Duration::minutes(5));
		cookies.add(cookie);
	}

	Ok(Template::render(&templates::Login { user: None }).into_response())
}

async fn login_post_handler(
	mut auth: AuthSession,
	cookies: Cookies,
	Form(login): Form<LoginCredentials>,
) -> RouteResult<HxResponse> {
	let next_url = cookies
		.get(LOGIN_NEXT_COOKIE_NAME)
		// avoid open redirections attacks
		.map(|ck| ck.value().to_owned())
		.filter(|ck| is_safe_relative_path(ck))
		.unwrap_or_else(|| "/".into());

	let Some(user) = auth.authenticate(login).await? else {
		let notices = HxNotices::new().add(
			"login-error",
			"could not authenticate using these credentials",
		);
		return Ok(notices.into_hx_response());
	};

	auth.login(&user).await?;

	Ok(HxRedirect::to(&next_url).into_hx_response())
}
