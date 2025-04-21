use std::borrow::Cow;

use askama::Template;
use axum::{
	Form, Router,
	http::HeaderValue,
	response::{Html, IntoResponse, Redirect, Response},
	routing::get,
};
use diesel::prelude::*;
use eyre::Context;
use reqwest::header::SET_COOKIE;
use serde::Deserialize;
use templates::{IndexTemplate, LoginTemplate};
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::{
	auth::{AuthSession, AuthnLayer, SessionSecret},
	config::{Ressources, RessourcesRef},
	database::{ResolvedUserEntry, ResolvedUserFeed, models::UserId},
	error::RouteResult,
};

mod templates;

pub fn router(ressources: &Ressources) -> Router<RessourcesRef> {
	let authn_layer = AuthnLayer::new(ressources);

	Router::new()
		.route("/", get(root_get_handler))
		.route("/profile", get(profile_get_handler))
		.route("/login", get(login_get_handler).post(login_post_handler))
		.nest("/web", web_fragment_router())
		.route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
		.nest_service("/static", ServeDir::new("static"))
		.layer(authn_layer)
}

pub fn web_fragment_router() -> Router<RessourcesRef> {
	Router::new().route("/", get(async || "Hello, FeedR!"))
}

async fn root_get_handler(auth: AuthSession, ressources: RessourcesRef) -> RouteResult<Response> {
	let user_id = auth.user_id_or_redirect()?;

	let mut conn = ressources.database_handle.get()?;
	let user_feeds = ResolvedUserFeed::resolve_all(user_id, &mut conn).unwrap();
	let user_entries = ResolvedUserEntry::resolve_all(user_id, &mut conn).unwrap();

	let template = IndexTemplate {
		username: format!("{user_id:?}").into(),
		user_feeds,
		user_entries,
	}
	.render();

	Ok(Html(template.unwrap()).into_response())
}

async fn profile_get_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
) -> RouteResult<Response> {
	let user_id = auth.user_id_or_redirect()?;

	let mut conn = ressources.database_handle.get()?;
	let user_feeds = ResolvedUserFeed::resolve_all(user_id, &mut conn).unwrap();
	let user_entries = ResolvedUserEntry::resolve_all(user_id, &mut conn).unwrap();

	let template = IndexTemplate {
		username: format!("{user_id:?}").into(),
		user_feeds,
		user_entries,
	}
	.render();

	Ok(Html(template.unwrap()).into_response())
}

async fn login_get_handler(auth: AuthSession) -> Response {
	let template = LoginTemplate {
		username: None,
		failed: false,
	}
	.render();

	Html(template.unwrap()).into_response()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
enum LoginInfo<'a> {
	#[serde(rename = "basic")]
	Basic {
		username: Cow<'a, str>,
		password: Cow<'a, str>,
	},
	#[serde(rename = "dauth")]
	DAuth { instance: Cow<'a, str> },
}

async fn login_post_handler(
	auth: AuthSession,
	ressources: RessourcesRef,
	Form(login): Form<LoginInfo<'_>>,
) -> RouteResult<Response> {
	match login {
		LoginInfo::Basic { username, password } => {
			use crate::database::schema::*;

			let mut conn = ressources.database_handle.get()?;

			let secret = user::table
				.select((user::id, user::tmp_unencrypted_secret))
				.filter(user::username.eq(&username))
				.get_result::<(UserId, Option<String>)>(&mut conn)
				.optional()
				.wrap_err("could not get user password")?;

			match secret {
				Some((user_id, Some(secret))) if secret == password => {
					let uuid = Uuid::new_v4().to_string();
					auth.session_map
						.lock()
						.insert(SessionSecret(uuid.clone()), user_id);

					// TODO: custom redir
					let mut res = Redirect::to("/").into_response();
					res.headers_mut().append(
						SET_COOKIE,
						HeaderValue::from_str(&format!("session={uuid}")).unwrap(),
					);
					Ok(res)
				}
				_ => {
					let template = LoginTemplate {
						username: None,
						failed: true,
					}
					.render()?;

					Ok(Html(template).into_response())
				}
			}
		}
		LoginInfo::DAuth { instance } => todo!(),
	}
}
