#[cfg(feature = "test-helpers")]
use fake::{Dummy, faker};

use crate::{Entity, Error, transaction};
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{FromRow, Sqlite, SqlitePool};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an invoice with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let invoices = sqlx::query_as(r#"SELECT * FROM
/// invoices where id = ?"#).bind(id).fetch_all(&pool).await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub amount: Option<f64>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

/// A changeset representing the data that is intended to be used to either create a new invoice or update an existing invoice.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let invoice_changeset: InvoiceChangeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct InvoiceChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "1.00..100.00"))]
    pub amount: Option<f64>,
}

/// The Entity trait implements all basic CRUD operations for the Invoice.
///
/// This allows us to GET | POST | PUT | DELETE invoices in our controllers.
///
/// ```
/// let invoice = Invoice::load(1, &pool).await?;
#[async_trait]
impl Entity for Invoice {
    type Id = Uuid;

    type Record<'a> = Invoice;

    type Changeset = InvoiceChangeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Invoice>, Error> {
        let invoices: Vec<Invoice> =
            sqlx::query_as(r#"select id, amount, created_at, updated_at from invoices"#)
                .fetch_all(executor)
                .await?;

        Ok(invoices)
    }

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Invoice, Error> {
        let invoice: Invoice = sqlx::query_as(
            r#"select id, amount, created_at, updated_at from invoices where id = ?"#,
        )
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(invoice)
    }

    async fn create<'a>(
        invoice: InvoiceChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Invoice, Error> {
        invoice.validate()?;

        let invoice: Invoice = sqlx::query_as(
            r#"insert into invoices (id, amount) values (?, ?) returning id, amount, created_at, updated_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(invoice.amount)
        .fetch_one(executor)
        .await?;

        Ok(invoice)
    }

    async fn create_batch(
        invoices: Vec<InvoiceChangeset>,
        pool: &SqlitePool,
    ) -> Result<Vec<Invoice>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Invoice> = vec![];

        for invoice in invoices {
            invoice.validate()?;

            let result = Invoice::create(invoice, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: Self::Id,
        invoice: InvoiceChangeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Invoice, Error> {
        invoice.validate()?;

        let invoice: Invoice = sqlx::query_as(
            r#"update invoices set (amount) = (?) where id = ? returning id, amount, created_at, updated_at"#,
        )
        .bind(invoice.amount)
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(invoice)
    }

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Invoice, Error> {
        let invoice: Invoice = sqlx::query_as(
            r#"delete from invoices where id = ? returning id, amount, created_at, updated_at"#,
        )
        .bind(id)
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok(invoice)
    }

    async fn delete_batch(ids: Vec<Self::Id>, pool: &SqlitePool) -> Result<Vec<Invoice>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<Invoice> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
