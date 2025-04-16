use std::borrow::Cow;

use askama::Template;
use axum::{
	Form, Router,
	http::HeaderValue,
	response::{Html, IntoResponse, Redirect, Response},
	routing::get,
};
use diesel::prelude::*;
use reqwest::header::SET_COOKIE;
use serde::Deserialize;
use templates::{IndexTemplate, LoginTemplate};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
	auth::{AuthSession, AuthnLayer},
	config::{Ressources, RessourcesRef},
	database::{ResolvedUserEntry, ResolvedUserFeed},
};

mod templates;

pub fn router(ressources: &Ressources) -> Router<RessourcesRef> {
	let authn_layer = AuthnLayer::new(ressources);

	Router::new()
		.route("/", get(root_get_handler))
		.route("/login", get(login_get_handler).post(login_post_handler))
		.nest("/web", web_fragment_router())
		.route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
		.nest_service("/static", ServeDir::new("static"))
		.layer(authn_layer)
}

pub fn web_fragment_router() -> Router<RessourcesRef> {
	Router::new().route("/", get(async || "Hello, FeedR!"))
}

async fn root_get_handler(auth: AuthSession, ressources: RessourcesRef) -> Response {
	let Some(user_id) = auth.user_id else {
		return Redirect::to("/login").into_response();
	};

	let mut conn = ressources.get_db_conn().unwrap();
	let user_feeds = ResolvedUserFeed::resolve_all(user_id, &mut conn).unwrap();
	let user_entries = ResolvedUserEntry::resolve_all(user_id, &mut conn).unwrap();

	let template = IndexTemplate {
		username: format!("{user_id:?}").into(),
		user_feeds,
		user_entries,
	}
	.render();

	Html(template.unwrap()).into_response()
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
) -> Response {
	match login {
		LoginInfo::Basic { username, password } => {
			let mut conn = ressources.get_db_conn().unwrap();

			use crate::database::schema::*;
			let secret = user::table
				.select(user::tmp_unencrypted_secret)
				.filter(user::username.eq(&username))
				.get_result::<Option<String>>(&mut conn)
				.optional()
				.unwrap();

			match secret {
				Some(Some(secret)) if secret == password => {
					// TODO: custom redir
					let mut res = Redirect::to("/").into_response();
					res.headers_mut()
						.append(SET_COOKIE, HeaderValue::from_str("session=abc").unwrap());
					res
				}
				_ => {
					let template = LoginTemplate {
						username: None,
						failed: true,
					}
					.render();

					Html(template.unwrap()).into_response()
				}
			}
		}
		LoginInfo::DAuth { instance } => todo!(),
	}
}
