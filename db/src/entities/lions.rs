#[cfg(feature = "test-helpers")]
use fake::{Dummy, faker};

use crate::{Entity, Error, transaction};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, Sqlite, SqlitePool, types::time::OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an lion with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let lions = sqlx::query_as!(
///     Lion,
///     r#"SELECT * FROM lions where id = ?"#,
///     id
///     )
///     .fetch_all(&pool)
///     .await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Lion {
    pub id: String,
    pub name: String,
    pub email: String,
}

/// A changeset representing the data that is intended to be used to either create a new lion or update an existing lion.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let lion_changeset: LionChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct LionChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    pub name: String,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    pub email: String,
}

/// The Entity trait implements all basic CRUD operations for the Lion.
///
/// This allows us to GET | POST | PUT | DELETE lions in our controllers.
///
/// ```
/// let lion = Lion::load(1, &pool).await?;
/// ```
#[async_trait]
impl Entity for Lion {
    type Id = String;

    type Record<'a> = Lion;

    type Changeset = LionChangeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Lion>, Error> {
        let lions = sqlx::query_as!(Lion, r#"select id, name, email from lions"#)
            .fetch_all(executor)
            .await?;

        Ok(lions)
    }

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Lion, Error> {
        let lion = sqlx::query_as!(
            Lion,
            r#"select id, name, email from lions where id = ?"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(lion)
    }

    async fn create<'a>(
        lion: LionChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Lion, Error> {
        lion.validate()?;

        let id = Uuid::now_v7().to_string();

        let lion = sqlx::query_as!(
            Lion,
            r#"insert into lions (id, name, email) values (?, ?, ?) returning id, name, email"#,
            id,
            lion.name,
            lion.email
        )
        .fetch_one(executor)
        .await?;

        Ok(lion)
    }

    async fn create_batch(
        lions: Vec<LionChangeset>,
        pool: &SqlitePool,
    ) -> Result<Vec<Lion>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Lion> = vec![];

        for lion in lions {
            lion.validate()?;

            let result = Lion::create(lion, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: Self::Id,
        lion: LionChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Lion, Error> {
        lion.validate()?;

        let lion = sqlx::query_as!(
            Lion,
            r#"update lions set (name, email) = (?, ?) where id = ? returning id, name, email"#,
            lion.name,
            lion.email,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(lion)
    }

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Lion, Error> {
        let lion = sqlx::query_as!(
            Lion,
            r#"delete from lions where id = ? returning id, name, email"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(lion)
    }

    async fn delete_batch(ids: Vec<Self::Id>, pool: &SqlitePool) -> Result<Vec<Lion>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Lion> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
