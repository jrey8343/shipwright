#[cfg(feature = "test-helpers")]
use fake::{faker, Dummy};

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use sqlx::{Sqlite, SqlitePool, FromRow, types::time::OffsetDateTime};
use uuid::Uuid;
use validator::Validate;
use crate::{Entity, Error, transaction};

/// A struct which maps the fields of an {{ entity_singular_name }} with native Sqlite types.
///
/// This allows you to use sqlx::query_as! to load records from the database and map them to this
/// struct.
///
/// ```
/// let {{ entity_plural_name }} = sqlx::query_as!(
///     {{ entity_struct_name }},
///     r#"SELECT * FROM {{ entity_plural_name }} where id = ?"#,
///     id
///     )
///     .fetch_all(&pool)
///     .await?;
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

/// The Entity trait implements all basic CRUD operations for the {{ entity_struct_name }}.
///
/// This allows us to GET | POST | PUT | DELETE {{ entity_plural_name }} in our controllers.
///
/// ```
/// let {{ entity_singular_name }} = {{ entity_struct_name }}::load(1, &pool).await?;
/// ``` 
#[async_trait]
impl Entity for {{ entity_struct_name }} {
    type Id = String;

    type Record<'a> = {{ entity_struct_name }};

    type Changeset = {{ entity_struct_name}}Changeset;

    async fn load_all<'a>(
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<{{ entity_struct_name }}>, Error> {
        let {{ entity_plural_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            r#"select {% for field in entity_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless%}{%- endfor %} from {{ entity_plural_name }}"#
        )
        .fetch_all(executor)
        .await?;

        Ok({{ entity_plural_name }})
    }

    async fn load<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<{{ entity_struct_name}}, Error> {
        let {{ entity_singular_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            r#"select {% for field in entity_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless %}{%- endfor %} from {{ entity_plural_name }} where id = ?"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok({{ entity_singular_name }})
    }

    async fn create<'a>(
        {{ entity_singular_name }}: {{ entity_struct_name }}Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<{{ entity_struct_name }}, Error> {
        {{ entity_singular_name }}.validate()?;

        let id = Uuid::now_v7().to_string();

        let {{ entity_singular_name }}  = sqlx::query_as!(
            {{ entity_struct_name }},
            r#"insert into {{ entity_plural_name }} (id, {% for field in changeset_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless %}{%- endfor %}) values (?, {% for field in changeset_struct_fields -%}?{% unless forloop.last %}, {% endunless %}{%- endfor %}) returning {% for field in entity_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless %}{%- endfor %}"#,
            id,
            {% for field in changeset_struct_fields -%}
            {{ entity_singular_name }}.{{ field.name }}{% unless forloop.last %},{% endunless %}
            {%- endfor %}
            )
            .fetch_one(executor)
            .await?;

        Ok({{ entity_singular_name }})
    }

    async fn create_batch(
        {{ entity_plural_name }}: Vec<{{ entity_struct_name }}Changeset>,
        pool: &SqlitePool,
    ) -> Result<Vec<{{ entity_struct_name }}>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<{{ entity_struct_name }}> = vec![];

        for {{ entity_singular_name }} in {{ entity_plural_name }} {
            {{ entity_singular_name }}.validate()?;

            let result = {{ entity_struct_name }}::create({{ entity_singular_name }}, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }

    async fn update<'a>(
        id: Self::Id,
        {{ entity_singular_name }}: {{ entity_struct_name }}Changeset,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<{{ entity_struct_name }}, Error> {
        {{ entity_singular_name }}.validate()?;

        let {{ entity_singular_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            r#"update {{ entity_plural_name }} set ({% for field in changeset_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless %}{%- endfor %}) = ({% for field in changeset_struct_fields -%}?{% unless forloop.last %}, {% endunless %}{%- endfor %}) where id = ? returning {% for field in entity_struct_fields -%}{{field.name}}{% unless forloop.last %}, {% endunless %}{%- endfor %}"#,
            {% for field in changeset_struct_fields -%}
            {{ entity_singular_name }}.{{ field.name }},
            {%- endfor %}
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok({{ entity_singular_name }})
    }

    async fn delete<'a>(
        id: Self::Id,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<{{ entity_struct_name }}, Error> {
        let {{ entity_singular_name }} = sqlx::query_as!(
            {{ entity_struct_name }},
            r#"delete from {{ entity_plural_name }} where id = ? returning {% for field in entity_struct_fields -%}{{ field.name }}{% unless forloop.last %}, {% endunless %}{%- endfor %}"#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::NoRecordFound)?;

        Ok({{ entity_singular_name }})
    }

    async fn delete_batch(ids: Vec<Self::Id>, pool: &SqlitePool) -> Result<Vec<{{ entity_struct_name }}>, Error> {
        let mut tx = transaction(pool).await?;

        let mut results: Vec<{{ entity_struct_name }}> = vec![];

        for id in ids {
            let result = Self::delete(id, &mut *tx).await?;
            results.push(result);
        }

        tx.commit().await?;

        Ok(results)
    }
}
