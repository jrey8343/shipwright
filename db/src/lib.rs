use std::borrow::Cow;

use async_trait::async_trait;
use shipwright_config::Config;
use sqlx::migrate::MigrateDatabase as _;
use sqlx::prelude::FromRow;
use sqlx::sqlite::SqliteRow;
use sqlx::{Sqlite, Transaction, sqlite::SqlitePoolOptions};

pub use serde::de::DeserializeOwned;
pub use sqlx::SqlitePool as DbPool;
pub use sqlx::test as db_test;
pub use validator::Validate;

/// Custom migrator set to the correct path within the api testing environment
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../db/migrations");

/// Entity definitions and related general queries.
pub mod entities;

#[derive(Default)]
pub enum Database {
    #[default]
    Primary,
    Jobs,
}

impl Database {
    pub fn to_url(&self, config: &Config) -> String {
        match self {
            Database::Primary => config.database.url.clone(),
            Database::Jobs => config.worker.database_url.clone(),
        }
    }
}
/// Starts a new database transaction.
///
/// Example:
/// ```
/// let tx = transaction(&app_state.db_pool).await?;
/// tasks::create(task_data, &mut *tx)?;
/// users::create(user_data, &mut *tx)?;
///
/// match tx.commit().await {
///     Ok(_) => Ok((StatusCode::CREATED, TasksView(results))),
///     Err(e) => Err((internal_error(e), "".into())),
/// }
/// ```
///
/// Transactions are rolled back automatically when they are dropped without having been committed.
pub async fn transaction(db_pool: &DbPool) -> Result<Transaction<'static, Sqlite>, Error> {
    let tx = db_pool.begin().await?;

    Ok(tx)
}

/// Creates a connection pool to the database specified in the passed [`{{project-name}}-config::DatabaseConfig`]
pub async fn connect_pool(database: Database, config: &Config) -> Result<DbPool, Error> {
    let pool = SqlitePoolOptions::new()
        .connect(&database.to_url(config))
        .await?;

    Ok(pool)
}

/// Create a database if it does not exist.
/// Used for parts of app where dbs are created
/// at runtime, e.g. tests, workers, tenants.
pub async fn create_database_if_not_exists(
    database: Database,
    config: &Config,
) -> Result<(), Error> {
    if !Sqlite::database_exists(&database.to_url(config)).await? {
        Sqlite::create_database(&database.to_url(config)).await?
    };
    Ok(())
}

/// Errors that can occur as a result of a data layer operation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// No record was found, e.g. when loading a record by ID. This variant is different from
    /// `Error::DbError(sqlx::Error::RowNotFound)` in that the latter indicates a bug, and
    /// `Error::NoRecordFound` does not. It merely originates from [sqlx::Executor::fetch_optional]
    /// returning `None`.
    #[error("no record found")]
    NoRecordFound,
    /// Return `422 Unprocessable Entity` on a unique constraint error.
    #[error("unique constraint error")]
    UniqueConstraint(Vec<(String, String)>),
    /// General database error, e.g. communicating with the database failed
    #[error("database query failed")]
    DatabaseError(#[from] sqlx::Error),
    /// An invalid changeset was passed to a writing operation such as creating or updating a record.
    #[error("validation failed")]
    ValidationError(#[from] validator::ValidationErrors),
    /// An error occurred while hashing a password.
    #[error("password hashing failed")]
    PasswordHashError(#[from] argon2::password_hash::Error),
}

/// ------------------------------------------------------------------------------------------
/// A little helper trait for more easily converting database constraint errors into API errors.
/// ------------------------------------------------------------------------------------------
/// ```rust,ignore
/// let user_id = sqlx::query_scalar!(
///     r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
///     username,
///     email,
///     password_hash
/// )
///     .fetch_one(&app_state.db)
///     .await
///     .on_constraint()?;
/// ```
pub trait ResultExt<T> {
    /// If `self` contains a SQLx database constraint error with the given name,
    /// transform the error.
    ///
    /// Otherwise, the result is passed through unchanged.
    fn map_constraint_err(self) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn map_constraint_err(self) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::DatabaseError(sqlx::Error::Database(dbe))
                if dbe.code() == Some(Cow::Borrowed("2067")) =>
            {
                let (_, field) = dbe
                    .message()
                    .strip_prefix("UNIQUE constraint failed: ") // strip down to table.field
                    .and_then(|s| s.split_once('.'))
                    .unwrap_or_default(); // return an empty string if parsing fails

                Error::UniqueConstraint(vec![(field.to_string(), dbe.message().to_string())])
            }
            e => e, // Pass the error through unchanged if not a sqlx error
        })
    }
}

/// ------------------------------------------------------------------------
/// # An Entity trait to implement common CRUD methods on a database table
/// ------------------------------------------------------------------------
///
/// Implement the Model trait on a specific model to get a full set
/// of common CRUD functions: list, show, create, update, delete
///
/// # Example
///
/// ```rust
/// #[async_trait]
/// impl Entity for Person {
///     type Id = i64;
///     type Record: Person;
///     type Changeset: PersonChangeset;
///
///     async fn list(db_pool: &DbPool) -> Result<Vec<Self::Record<'_>>, Error> {
///         // your implementation here
///         Ok(vec![])
///         }
///     // ...other methods
/// ```
///
/// ------------------------------------------------------------------------
#[async_trait]
pub trait Entity {
    type Id: PartialOrd;
    type Record<'a>: FromRow<'a, SqliteRow>;
    type Changeset: Validate + DeserializeOwned;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Self::Record<'a>>, Error>;

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn create<'a>(
        record: Self::Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn create_batch(
        records: Vec<Self::Changeset>,
        db_pool: &DbPool,
    ) -> Result<Vec<Self::Record<'_>>, Error>;

    async fn update<'a>(
        id: Self::Id,
        record: Self::Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Self::Record<'a>, Error>;

    async fn delete_batch(
        keys: Vec<Self::Id>,
        db_pool: &DbPool,
    ) -> Result<Vec<Self::Record<'_>>, Error>;
}
