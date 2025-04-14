#[cfg(feature = "test-helpers")]
use fake::{Dummy, faker};
use time::OffsetDateTime;

use crate::{Entity, Error, transaction};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, Sqlite, SqlitePool};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an kid with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let kids = sqlx::query_as!(
///     Kid,
///     r#"SELECT * FROM kids where id = ?"#,
///     id
///     )
///     .fetch_all(&pool)
///     .await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Kid {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub nickname: String,
    pub favourite_paw_patrol: String,
}

/// A changeset representing the data that is intended to be used to either create a new kid or update an existing kid.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let kid_changeset: KidChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct KidChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    pub name: String,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    pub nickname: String,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    pub favourite_paw_patrol: String,

    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::time::en::DateTime()"))]
    pub updated_at: OffsetDateTime,
}

/// The Entity trait implements all basic CRUD operations for the Kid.
///
/// This allows us to GET | POST | PUT | DELETE kids in our controllers.
///
/// ```
/// let kid = Kid::load(1, &pool).await?;
/// ```
#[async_trait]
impl Entity for Kid {
    type Id = String;

    type Record<'a> = Kid;

    type Changeset = KidChangeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Kid>, Error> {
        let kids = sqlx::query_as!(
            Kid,
            r#"select id, created_at, updated_at, name, nickname, favourite_paw_patrol from kids"#
        )
        .fetch_all(executor)
        .await?;

        Ok(kids)
    }

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Kid, Error> {
        let kid = sqlx::query_as!(
            Kid,
            r#"select id, created_at, updated_at, name, nickname, favourite_paw_patrol from kids where id = ?"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(kid)
    }

    async fn create<'a>(
        kid: KidChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Kid, Error> {
        kid.validate()?;

        let id = Uuid::now_v7().to_string();

        let kid  = sqlx::query_as!(
            Kid,
            r#"insert into kids (id, name, nickname, favourite_paw_patrol) values (?, ?, ?, ?) returning id, created_at, updated_at, name, nickname, favourite_paw_patrol"#,
            id,
            kid.name,
            kid.nickname,
            kid.favourite_paw_patrol
            )
            .fetch_one(executor)
            .await?;

        Ok(kid)
    }

    async fn create_batch(kids: Vec<KidChangeset>, pool: &SqlitePool) -> Result<Vec<Kid>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Kid> = vec![];

        for kid in kids {
            kid.validate()?;

            let result = Kid::create(kid, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: Self::Id,
        kid: KidChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Kid, Error> {
        kid.validate()?;

        let kid = sqlx::query_as!(
            Kid,
            r#"update kids set (updated_at, name, nickname, favourite_paw_patrol) = (?, ?, ?, ?) where id = ? returning id, created_at, updated_at, name, nickname, favourite_paw_patrol"#,
            kid.updated_at,
            kid.name,
            kid.nickname,
            kid.favourite_paw_patrol,
            id,
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(kid)
    }

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Kid, Error> {
        let kid = sqlx::query_as!(
            Kid,
            r#"delete from kids where id = ? returning id, created_at, updated_at, name, nickname, favourite_paw_patrol"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(kid)
    }

    async fn delete_batch(ids: Vec<Self::Id>, pool: &SqlitePool) -> Result<Vec<Kid>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Kid> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
