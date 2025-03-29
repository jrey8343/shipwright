use color_eyre::eyre::eyre;
use cruet::to_plural;
use sea_query::{Alias, ColumnDef, Expr};
use serde::Serialize;

use crate::Error;

#[derive(Debug, Clone)]
pub enum Field {
    Column(String, FieldType),
    ForeignKey {
        local_key: String,
        references_table: String,
        references_column: String,
    },
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Uuid {
        nullable: bool,
        unique: bool,
    },
    String {
        nullable: bool,
        unique: bool,
        text: bool,
        length: Option<u32>,
    },
    Integer {
        nullable: bool,
        unique: bool,
        size: IntegerSize,
    },
    Float {
        nullable: bool,
        unique: bool,
    },
    Double {
        nullable: bool,
        unique: bool,
    },
    Decimal {
        nullable: bool,
        unique: bool,
    },
    Boolean {
        nullable: bool,
    },
    Date,
    DateTime,
    Json {
        binary: bool,
        unique: bool,
    },
}

#[derive(Debug, Clone)]
pub enum IntegerSize {
    Small,
    Regular,
    Big,
    Unsigned,
}

impl FieldType {
    pub fn from_compact_type(input: &str) -> Option<FieldType> {
        let base = input
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>();

        let rest = &input[base.len()..];

        let mut length_digits = String::new();
        let mut nullable = true;
        let mut unique = false;

        for c in rest.chars() {
            if c.is_ascii_digit() {
                length_digits.push(c);
            } else if c == '!' {
                nullable = false;
            } else if c == '^' {
                unique = true;
            }
        }

        let length = if length_digits.is_empty() {
            None
        } else {
            length_digits.parse::<u32>().ok()
        };

        match base.as_str() {
            "string" => Some(FieldType::String {
                nullable,
                unique,
                text: false,
                length,
            }),
            "text" => Some(FieldType::String {
                nullable,
                unique,
                text: true,
                length: None,
            }),
            "uuid" => Some(FieldType::Uuid { nullable, unique }),
            "int" => Some(FieldType::Integer {
                nullable,
                unique,
                size: IntegerSize::Regular,
            }),
            "bigint" => Some(FieldType::Integer {
                nullable,
                unique,
                size: IntegerSize::Big,
            }),
            "smallint" => Some(FieldType::Integer {
                nullable,
                unique,
                size: IntegerSize::Small,
            }),
            "unsigned" => Some(FieldType::Integer {
                nullable,
                unique,
                size: IntegerSize::Unsigned,
            }),
            "float" => Some(FieldType::Float { nullable, unique }),
            "double" => Some(FieldType::Double { nullable, unique }),
            "decimal" => Some(FieldType::Decimal { nullable, unique }),
            "bool" => Some(FieldType::Boolean { nullable }),
            "date" => Some(FieldType::Date),
            "datetime" => Some(FieldType::DateTime),
            "json" => Some(FieldType::Json {
                binary: false,
                unique,
            }),
            "jsonb" => Some(FieldType::Json {
                binary: true,
                unique,
            }),
            _ => None,
        }
    }

    pub fn to_column_def(&self, name: &str) -> ColumnDef {
        let mut col = ColumnDef::new(Alias::new(name));

        match self {
            FieldType::Uuid { nullable, unique } => {
                col.uuid();
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::String {
                nullable,
                unique,
                text,
                length,
            } => {
                if *text {
                    col.text();
                } else if let Some(len) = length {
                    col.string_len(*len);
                } else {
                    col.string();
                }
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::Integer {
                nullable, unique, ..
            } => {
                col.integer();
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::Float { nullable, unique } => {
                col.float();
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::Double { nullable, unique } => {
                col.double();
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::Decimal { nullable, unique } => {
                col.decimal();
                if !nullable {
                    col.not_null();
                }
                if *unique {
                    col.unique_key();
                }
            }
            FieldType::Boolean { nullable } => {
                col.boolean();
                if !nullable {
                    col.not_null();
                }
            }
            FieldType::Date => {
                col.date().not_null();
            }
            FieldType::DateTime => {
                col.date_time()
                    .not_null()
                    .default(Expr::cust("CURRENT_TIMESTAMP"));
            }
            FieldType::Json { binary, unique } => {
                if *binary {
                    col.json_binary();
                } else {
                    col.json();
                }
                col.not_null();
                if *unique {
                    col.unique_key();
                }
            }
        }

        col
    }

    pub fn as_sqlx_type(&self) -> String {
        match self {
            FieldType::Uuid { nullable, .. } => {
                if *nullable { "Option<Uuid>" } else { "Uuid" }.into()
            }
            FieldType::String { nullable, .. } => if *nullable {
                "Option<String>"
            } else {
                "String"
            }
            .into(),
            FieldType::Integer { nullable, .. } => {
                if *nullable { "Option<i64>" } else { "i64" }.into()
            }
            FieldType::Float { nullable, .. } => {
                if *nullable { "Option<f32>" } else { "f32" }.into()
            }
            FieldType::Double { nullable, .. } => {
                if *nullable { "Option<f64>" } else { "f64" }.into()
            }
            FieldType::Decimal { nullable, .. } => if *nullable {
                "Option<Decimal>"
            } else {
                "Decimal"
            }
            .into(),
            FieldType::Boolean { nullable } => {
                if *nullable { "Option<bool>" } else { "bool" }.into()
            }
            FieldType::Date | FieldType::DateTime => "time::OffsetDateTime".into(),
            FieldType::Json { .. } => "serde_json::JsonValue".into(),
        }
    }

    pub fn as_faker(&self) -> Option<String> {
        match self {
            FieldType::String { .. } => Some("faker::name::en::Name()".to_owned()),
            FieldType::Uuid { .. } => Some("faker::uuid::UUIDv4::Uuid()".to_owned()),
            FieldType::Integer { .. } => Some("1..100".to_owned()),
            FieldType::Float { .. } => Some("1.0..100.0".to_owned()),
            FieldType::Double { .. } => Some("1.00..100.00".to_owned()),
            FieldType::Boolean { .. } => Some("faker::boolean::en::Boolean()".to_owned()),
            FieldType::Date | FieldType::DateTime => Some("faker::time::en::DateTime()".to_owned()),
            _ => None,
        }
    }

    pub fn as_validation(&self) -> Option<String> {
        match self {
            FieldType::String { length, .. } => {
                if let Some(len) = length {
                    Some(format!("length(min = 1, max = {})", len))
                } else {
                    Some("length(min = 1)".to_string())
                }
            }
            _ => None,
        }
    }
}

// rest of your code remains unchanged
pub fn parse_cli_fields(raw_fields: Vec<String>) -> Result<Vec<Field>, Error> {
    let mut fields = vec![];

    for field in raw_fields {
        let mut parts = field.splitn(2, ':');
        let name = parts
            .next()
            .ok_or_else(|| eyre!("Missing column name in: {}", field))?;
        let spec = parts
            .next()
            .ok_or_else(|| eyre!("Missing type spec in: {}", field))?;

        if spec.starts_with("references") {
            let local_key = format!("{}_id", name);
            let (references_table, references_column) =
                if let Some(ref_part) = spec.strip_prefix("references=") {
                    let (table, col) = ref_part
                        .split_once('(')
                        .map(|(t, rest)| {
                            let col = rest.trim_end_matches(')');
                            (t.to_string(), col.to_string())
                        })
                        .ok_or_else(|| eyre!("Invalid foreign key format: {}", spec))?;
                    (table, col)
                } else {
                    (to_plural(name), "id".to_string()) // default to plural table name
                };

            fields.push(Field::ForeignKey {
                local_key,
                references_table,
                references_column,
            });
        } else {
            let field_type = FieldType::from_compact_type(spec)
                .ok_or_else(|| eyre!("Invalid type specifier: {}", spec))?;
            fields.push(Field::Column(name.to_string(), field_type));
        }
    }

    Ok(fields)
}

pub async fn generate_sql(table_name: &str, fields: Vec<Field>) -> Result<String, Error> {
    let mut table = sea_query::Table::create();
    table.table(Alias::new(table_name)).if_not_exists();

    for field in fields {
        match field {
            Field::Column(name, field_type) => {
                let col = field_type.to_column_def(&name);
                table.col(col);
            }
            Field::ForeignKey {
                local_key,
                references_table,
                references_column,
            } => {
                table.col(ColumnDef::new(Alias::new(&local_key)).integer().not_null());

                table.foreign_key(
                    sea_query::ForeignKey::create()
                        .from(Alias::new(table_name), Alias::new(&local_key))
                        .to(
                            Alias::new(&references_table),
                            Alias::new(&references_column),
                        )
                        .on_delete(sea_query::ForeignKeyAction::Cascade)
                        .on_update(sea_query::ForeignKeyAction::Cascade),
                );
            }
        }
    }

    let sql = table.to_string(sea_query::SqliteQueryBuilder);
    Ok(sql)
}

#[derive(Debug, Clone, Serialize)]
pub struct StructField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangesetField {
    pub name: String,
    pub ty: String,
    pub validation: Option<String>,
    pub faker: Option<String>,
}

pub fn generate_struct_fields(fields: &[Field]) -> (Vec<StructField>, Vec<ChangesetField>) {
    let mut struct_fields = vec![];
    let mut changeset_fields = vec![];

    for field in fields {
        match field {
            Field::Column(name, field_type) => {
                let ty = field_type.as_sqlx_type();

                // Always include in the main struct
                struct_fields.push(StructField {
                    name: name.clone(),
                    ty: ty.clone(),
                });

                // Skip `id` in changeset
                if name != "id" {
                    changeset_fields.push(ChangesetField {
                        name: name.clone(),
                        ty,
                        validation: field_type.as_validation(),
                        faker: field_type.as_faker(),
                    });
                }
            }

            Field::ForeignKey { local_key, .. } => {
                // Foreign keys are always Uuid
                struct_fields.push(StructField {
                    name: local_key.clone(),
                    ty: "Uuid".to_string(),
                });

                changeset_fields.push(ChangesetField {
                    name: local_key.clone(),
                    ty: "Uuid".to_string(),
                    validation: None,
                    faker: Some("Uuid()".to_string()),
                });
            }
        }
    }

    (struct_fields, changeset_fields)
}
