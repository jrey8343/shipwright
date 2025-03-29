#[cfg(feature = "test-helpers")]
use fake::{faker, Dummy};

use serde::Deserialize;
use serde::Serialize;
use sqlx::{Sqlite, FromRow};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an invoice with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let invoices = sqlx::query_as!(Invoice, r#"SELECT * FROM
/// invoices"#).fetch_all(&pool).await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub amount: Option<f32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    
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
    #[cfg_attr(feature = "test-helpers", dummy(faker = "1.0..100.0"))]
    pub amount: Option<f32>,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::chrono::en::DateTime()"))]
    pub created_at: chrono::NaiveDateTime,
    #[cfg_attr(feature = "test-helpers", dummy(faker = "faker::chrono::en::DateTime()"))]
    pub updated_at: chrono::NaiveDateTime,
    
}
