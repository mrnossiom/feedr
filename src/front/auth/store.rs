use diesel::{dsl, prelude::*, r2d2::PoolError, result::DatabaseErrorKind};
use tower_sessions::{
	ExpiredDeletion, SessionStore,
	session::{Id, Record},
	session_store,
};

use crate::database::{
	PoolConnection, PooledConnection,
	models::{Session, SessionSecret},
};

#[derive(Debug, Clone)]
pub struct SqliteStore {
	db_pool: PoolConnection,
}

impl SqliteStore {
	pub const fn new(db_pool: PoolConnection) -> Self {
		Self { db_pool }
	}

	fn conn(&self) -> session_store::Result<PooledConnection> {
		self.db_pool
			.get()
			.map_err(|err| session_store::Error::Backend(err.to_string()))
	}

	fn try_insert_with_conn(
		conn: &mut PooledConnection,
		record: &Record,
	) -> Result<bool, SqliteStoreError> {
		use crate::database::schema::session;

		let data = rmp_serde::encode::to_vec(&record)?;

		let stmt = Session {
			id: SessionSecret(record.id.to_string()),
			data: &data,
			expiry_date: record.expiry_date,
		}
		.insert_into(session::table);

		match stmt.execute(conn) {
			Ok(1) => Ok(true),
			Ok(_)
			| Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(false),
			Err(err) => Err(err.into()),
		}
	}

	fn upsert_with_conn(
		conn: &mut PooledConnection,
		record: &Record,
	) -> Result<(), SqliteStoreError> {
		use crate::database::schema::session;

		let data = rmp_serde::encode::to_vec(&record.data)?;

		let stmt = Session {
			id: SessionSecret(record.id.to_string()),
			data: &data,
			expiry_date: record.expiry_date,
		}
		.insert_into(session::table)
		.on_conflict(session::id)
		.do_update()
		.set((
			session::data.eq(&data),
			session::expiry_date.eq(record.expiry_date),
		));

		stmt.execute(conn)?;
		Ok(())
	}
}

#[async_trait::async_trait]
impl ExpiredDeletion for SqliteStore {
	async fn delete_expired(&self) -> session_store::Result<()> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;

		dsl::delete(session::table)
			.filter(session::expiry_date.lt(dsl::now))
			.execute(&mut conn)
			.map_err(SqliteStoreError::Diesel)?;

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SqliteStoreError {
	#[error("rmp decode: {0}")]
	RmpDecode(#[from] rmp_serde::decode::Error),

	#[error("rmp encode: {0}")]
	RmpEncode(#[from] rmp_serde::encode::Error),

	#[error("pool: {0}")]
	DbPool(#[from] PoolError),

	#[error("diesel: {0}")]
	Diesel(#[from] diesel::result::Error),
}

impl From<SqliteStoreError> for session_store::Error {
	fn from(value: SqliteStoreError) -> Self {
		match value {
			SqliteStoreError::RmpDecode(err) => Self::Decode(err.to_string()),
			SqliteStoreError::RmpEncode(err) => Self::Encode(err.to_string()),
			SqliteStoreError::DbPool(err) => Self::Backend(err.to_string()),
			SqliteStoreError::Diesel(err) => Self::Backend(err.to_string()),
		}
	}
}

#[async_trait::async_trait]
impl SessionStore for SqliteStore {
	async fn create(&self, record: &mut Record) -> session_store::Result<()> {
		let mut conn = self.conn()?;
		conn.transaction::<_, SqliteStoreError, _>(|conn| {
			while !Self::try_insert_with_conn(conn, record)? {
				// Generate a new ID
				record.id = Id::default();
			}

			Ok(())
		})?;

		Ok(())
	}

	async fn save(&self, record: &Record) -> session_store::Result<()> {
		let mut conn = self.conn()?;
		Self::upsert_with_conn(&mut conn, record)?;
		Ok(())
	}

	async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;
		let stmt = session::table.select(session::data).filter(
			session::id
				.eq(session_id.to_string())
				.and(session::expiry_date.gt(dsl::now)),
		);

		let data = stmt
			.get_result::<Vec<u8>>(&mut conn)
			.optional()
			.map_err(SqliteStoreError::Diesel)?;

		if let Some(data) = data {
			let data = rmp_serde::from_slice(&data).map_err(SqliteStoreError::RmpDecode)?;
			Ok(Some(data))
		} else {
			Ok(None)
		}
	}

	async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;
		dsl::delete(session::table)
			.filter(session::id.eq(session_id.to_string()))
			.execute(&mut conn)
			.map_err(SqliteStoreError::Diesel)?;
		Ok(())
	}
}
