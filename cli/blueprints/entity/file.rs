#[cfg(feature = "test-helpers")]
use fake::{faker, Dummy};

use serde::Deserialize;
use serde::Serialize;
use sqlx::{Sqlite, FromRow};
use uuid::Uuid;
use validator::Validate;

/// A struct which maps the fields of an {{ entity_singular_name }} with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let {{ entity_plural_name }} = sqlx::query_as!({{ entity_struct_name }}, r#"SELECT * FROM
/// {{ entity_plural_name }}"#).fetch_all(&pool).await?;
/// ```
#[derive(Serialize, Debug, Deserialize, FromRow)]
pub struct {{entity_struct_name}} {
    {% for field in entity_struct_fields -%}
    pub {{ field.name }}: {{ field.ty }},
    {% endfor %}
}

/// A changeset representing the data that is intended to be used to either create a new {{ entity_singular_name }} or update an existing {{ entity_singular_name }}.
///
/// Changesets are validatated in the [`create`] and [`update`] functions which return an [Result::Err] if validation fails.
///
/// Changesets can also be used to generate fake data for tests when the `test-helpers` feature is enabled:
///
/// ```
/// let {{ entity_singular_name }}_changeset: {{ entity_struct_name }}Changeset = Faker.fake();
/// ```
#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct {{entity_struct_name}}Changeset {
    {% for field in changeset_struct_fields -%}
    {% if field.faker -%}
    #[cfg_attr(feature = "test-helpers", dummy(faker = "{{ field.faker }}"))]
    {%- endif %}
    pub {{ field.name }}: {{ field.ty }},
    {% endfor %}
}
