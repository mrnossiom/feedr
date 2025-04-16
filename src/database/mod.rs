use diesel::{
	Connection, ExpressionMethods, OptionalExtension, QueryDsl, QueryResult, RunQueryDsl,
	SqliteConnection, dsl::insert_into, r2d2,
};
use url::Url;

use self::models::{Feed, FeedId, UserId};

#[rustfmt::skip]
pub mod schema;
pub mod models;

pub type PooledConnection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

impl UserId {
	pub const fn new(id: i32) -> Self {
		Self(id)
	}
}

impl Feed<'_> {
	pub fn resolve_or_create(url: &Url, conn: &mut PooledConnection) -> QueryResult<FeedId> {
		conn.transaction(|conn| {
			use crate::database::schema::*;
			let id = feed::table
				.select(feed::id)
				.filter(feed::url.eq(url.as_str()))
				.get_result::<FeedId>(conn)
				.optional()?;

			id.map_or_else(
				|| {
					insert_into(feed::table)
						.values((feed::url.eq(url.as_str()), feed::status.eq(1)))
						.returning(feed::id)
						.get_result(conn)
				},
				Ok,
			)
		})
	}
}
