use core::fmt;
use std::{collections::BTreeSet, usize};

use getset::Getters;
use sea_query::{ColumnDef, Iden, Table, TableCreateStatement};
use serde::{
    de::{Unexpected, Visitor},
    Deserialize, Serialize,
};
use thiserror::Error;

#[cfg(test)]
mod tests;

pub struct IdenString(pub String);

impl IdenString {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Iden for IdenString {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        write!(s, "{}", &self.0.to_lowercase()).unwrap();
    }
}

#[macro_export]
macro_rules! iden_str {
    ($table_name: ident) => {
        IdenString(String::from($table_name))
    };
    ($table_name: expr) => {
        IdenString($table_name.into())
    };
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Type {
    #[serde(rename = "integer")]
    Integer,

    #[serde(rename = "float")]
    Float,

    #[serde(rename = "text")]
    Text,

    #[serde(rename = "bool")]
    Bool,
}

#[derive(Debug, Error)]
pub enum TypeErrors {
    #[error("Could not convert the given type")]
    UnimplementedConversion,
}

impl<'a> TryFrom<&'a serde_json::Value> for Type {
    type Error = TypeErrors;

    fn try_from(value: &'a serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null
            | serde_json::Value::Array(_)
            | serde_json::Value::Object(_) => Err(TypeErrors::UnimplementedConversion),
            serde_json::Value::Number(n) => Ok({
                if n.is_i64() {
                    Type::Integer
                } else {
                    Type::Float
                }
            }),
            serde_json::Value::String(_) => Ok(Type::Text),
            serde_json::Value::Bool(_) => Ok(Type::Bool),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Getters, Eq)]
#[getset(get = "pub")]
pub struct Field {
    name: String,
    #[serde(rename = "type")]
    field_type: Type,
    #[serde(default)]
    nullable: bool,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LiveSchema(Vec<Option<(Field, serde_json::Value)>>);

impl LiveSchema {
    pub fn new(_capacity: usize) -> Self {
        Self(Vec::with_capacity(_capacity))
    }

    fn inner(&self) -> &[Option<(Field, serde_json::Value)>] {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut Vec<Option<(Field, serde_json::Value)>> {
        &mut self.0
    }

    /// Generates a create table statement using Seaquery (part of SeaORM), this statement
    /// is backend agnostic, the translation to a specific flavor of SQL is done with a
    /// QueryBuilder, the query builder _used for testing_ is the
    /// [PostgresQueryBuilder](sea_query::PostgresQueryBuilder).
    pub fn table_create_statement<'a>(&self, table_name: &'a str) -> TableCreateStatement {
        // The table create statement is done using a constructor that is builder like.
        let mut statement = Table::create();
        // The iden_str! macro here, allows us to provide a runtime String, as the table name
        statement.table(iden_str!(table_name));

        // Go through each Field in the vec and create a corresponding column for it
        for entry in self.inner().iter() {
            if entry.is_none() {
                continue;
            }
            let (entry, _) = entry.as_ref().unwrap();
            let mut column = ColumnDef::new(iden_str!(entry.name()));

            entry.nullable().then(|| column.null());

            let col_type = match entry.field_type() {
                Type::Integer => column.integer(),
                Type::Float => column.float(),
                Type::Text => column.text(),
                Type::Bool => column.boolean(),
            };
            statement.col(col_type);
        }

        let mut table_unique_id = ColumnDef::new(iden_str!("id"));
        table_unique_id.integer().not_null().auto_increment();

        statement.col(table_unique_id.primary_key()).to_owned()
    }
}

/// A **Schema** is an abstraction placed bettwen the JSON schema,
/// and the adequeate SQL syntax to represent said schema, as a table.
/// Right now, a schema supports only data types present in the _enum_ [Type]
#[derive(Debug, Default, Serialize)]
pub struct Schema(Vec<Option<Field>>);

impl Schema {
    pub fn inner(&self) -> &[Option<Field>] {
        &self.0
    }
    fn inner_mut(&mut self) -> &mut [Option<Field>] {
        &mut self.0
    }

    /// Generates a create table statement using Seaquery (part of SeaORM), this statement
    /// is backend agnostic, the translation to a specific flavor of SQL is done with a
    /// QueryBuilder, the query builder _used for testing_ is the
    /// [PostgresQueryBuilder](sea_query::PostgresQueryBuilder).
    pub fn table_create_statement<'a>(&self, table_name: &'a str) -> TableCreateStatement {
        // The table create statement is done using a constructor that is builder like.
        let mut statement = Table::create();
        // The iden_str! macro here, allows us to provide a runtime String, as the table name
        statement.table(iden_str!(table_name));

        // Go through each Field in the vec and create a corresponding column for it
        for entry in self.inner().iter() {
            if entry.is_none() {
                continue;
            }
            let entry = entry.as_ref().unwrap();
            let mut column = ColumnDef::new(iden_str!(entry.name()));

            entry.nullable().then(|| column.null());

            let col_type = match entry.field_type() {
                Type::Integer => column.integer(),
                Type::Float => column.float(),
                Type::Text => column.text(),
                Type::Bool => column.boolean(),
            };
            statement.col(col_type);
        }

        let mut table_unique_id = ColumnDef::new(iden_str!("id"));
        table_unique_id.integer().not_null().auto_increment();

        statement.col(table_unique_id.primary_key()).to_owned()
    }
}

// Start section --- Custom serde impls

/// We always expect the Schema to be a sequence (array) of fields
impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(SchemaVisitor)
    }
}

/// The actual behaviour for deserializing a Schema using serde
struct SchemaVisitor;

impl<'de> Visitor<'de> for SchemaVisitor {
    type Value = Schema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an invalid Schema declaration.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut existing = BTreeSet::<String>::new();
        let mut schema = Schema::default();
        let mut i = 0;

        while let Ok(Some(entry)) = seq.next_element::<Field>() {
            if existing.contains(&entry.name) {
                Err(serde::de::Error::duplicate_field("Duplicate Field"))?;
            };
            existing.insert(entry.name.clone());
            schema.inner_mut().get(i).replace(&mut Some(entry));
            i += 1;
        }

        Ok(schema)
    }
}

/// The actual behaviour for deserializing a LiveSchema using serde
struct LiveSchemaVisitor;

impl<'de> Visitor<'de> for LiveSchemaVisitor {
    type Value = LiveSchema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an invalid Field declaration.")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let field_count_guess = map.size_hint().unwrap_or(1);
        let mut live_schema = LiveSchema::new(field_count_guess);

        while let Some((key, value)) = map.next_entry()? {
            let value: serde_json::Value = value;
            let field = Field {
                name: key,
                field_type: Type::try_from(&value).map_err(|_| {
                    serde::de::Error::invalid_type(
                        Unexpected::Other("unimplemented conversion for given type"),
                        &self,
                    )
                })?,
                nullable: false,
            };

            live_schema.inner_mut().push(Some((field, value)));
        }

        live_schema.0.shrink_to_fit();
        Ok(live_schema)
    }
}

impl<'de> Deserialize<'de> for LiveSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(LiveSchemaVisitor)
    }
}

// End section --- Custom serde impls
