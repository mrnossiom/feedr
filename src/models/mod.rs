use std::borrow::Cow;

use diesel::prelude::Insertable;

#[rustfmt::skip]
pub mod schema;

#[derive(Insertable)]
#[diesel(table_name = schema::user)]
struct User<'a> {
	id: i32,

	username: Cow<'a, str>,

	d_auth_secret: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::api_key)]
struct ApiKey<'a> {
	id: i32,
	user_id: i32,

	name: Cow<'a, str>,

	secret: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::feed)]
struct Feed<'a> {
	id: i32,

	url: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::feed_entry)]
struct FeedEntry<'a> {
	id: i32,

	feed_id: i32,
	user_id: i32,

	title: Cow<'a, str>,
	description: Cow<'a, str>,
}
