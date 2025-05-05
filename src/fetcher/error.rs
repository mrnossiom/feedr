pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("pool: {0}")]
	DbPool(#[from] diesel::r2d2::PoolError),

	#[error("other: {0}")]
	Other(#[from] eyre::Report),
}
