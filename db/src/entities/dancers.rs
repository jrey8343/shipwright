#[cfg(feature = "test-helpers")]
use fake::{Dummy, faker};

use crate::{Entity, Error, transaction};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, Sqlite, SqlitePool};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an dancer with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let dancers = sqlx::query_as!(
///     Dancer,
///     r#"SELECT * FROM dancers where id = ?"#,
///     id
///     )
///     .fetch_all(&pool)
///     .await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Dancer {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub email: String,
    pub dance_style: String,
}

/// A changeset representing the data that is intended to be used to either create a new dancer or update an existing dancer.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let dancer_changeset: DancerChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct DancerChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::name::en::Name()"))]
    #[validate(length(min = 1))]
    pub name: String,
    #[cfg_attr(
        feature = "test-helpers",
        dummy(faker = "faker::internet::en::SafeEmail()")
    )]
    #[validate(email)]
    pub email: String,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::lorem::en::Word()"))]
    #[validate(length(min = 1))]
    pub dance_style: String,
}

/// The Entity trait implements all basic CRUD operations for the Dancer.
///
/// This allows us to GET | POST | PUT | DELETE dancers in our controllers.
///
/// ```
/// let dancer = Dancer::load(1, &pool).await?;
/// ```
#[async_trait]
impl Entity for Dancer {
    type Id = String;

    type Record<'a> = Dancer;

    type Changeset = DancerChangeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Dancer>, Error> {
        let dancers = sqlx::query_as!(
            Dancer,
            r#"select id, created_at, updated_at, name, email, dance_style from dancers"#
        )
        .fetch_all(executor)
        .await?;

        Ok(dancers)
    }

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Dancer, Error> {
        let dancer = sqlx::query_as!(
            Dancer,
            r#"select id, created_at, updated_at, name, email, dance_style from dancers where id = ?"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(dancer)
    }

    async fn create<'a>(
        dancer: DancerChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Dancer, Error> {
        dancer.validate()?;

        let id = Uuid::now_v7().to_string();

        let dancer  = sqlx::query_as!(
            Dancer,
            r#"insert into dancers (id, name, email, dance_style) values (?, ?, ?, ?) returning id, created_at, updated_at, name, email, dance_style"#,
            id,
            dancer.name,dancer.email,dancer.dance_style
            )
            .fetch_one(executor)
            .await?;

        Ok(dancer)
    }

    async fn create_batch(
        dancers: Vec<DancerChangeset>,
        pool: &SqlitePool,
    ) -> Result<Vec<Dancer>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Dancer> = vec![];

        for dancer in dancers {
            dancer.validate()?;

            let result = Dancer::create(dancer, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: Self::Id,
        dancer: DancerChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Dancer, Error> {
        dancer.validate()?;

        let dancer = sqlx::query_as!(
            Dancer,
            r#"update dancers set (name, email, dance_style) = (?, ?, ?) where id = ? returning id, created_at, updated_at, name, email, dance_style"#,
            dancer.name,dancer.email,dancer.dance_style,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(dancer)
    }

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Dancer, Error> {
        let dancer = sqlx::query_as!(
            Dancer,
            r#"delete from dancers where id = ? returning id, created_at, updated_at, name, email, dance_style"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(dancer)
    }

    async fn delete_batch(ids: Vec<Self::Id>, pool: &SqlitePool) -> Result<Vec<Dancer>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Dancer> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
