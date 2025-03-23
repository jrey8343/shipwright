use color_eyre::eyre::eyre;
use cruet::to_plural;
use sea_query::{Alias, ColumnDef};

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
                    col.string(); // fallback
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
                col.date_time().not_null();
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
}

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
