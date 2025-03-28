#[cfg(feature = "test-helpers")]
use fake::{faker::name::en::*, Dummy};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Sqlite;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Deserialize)]
pub struct {{entity_struct_name}} {
    {% for field in entity_struct_fields -%}
    pub {{ field.name }}: {{ field.ty }},
    {% endfor %}
}

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct {{entity_struct_name}}Changeset {
    {% for field in changeset_struct_fields -%}
    pub {{ field.name }}: {{ field.ty }},
    {% endfor %}
}
