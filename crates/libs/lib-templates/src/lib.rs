use core::fmt;
use std::collections::HashSet;

use getset::Getters;
use sea_query::{ColumnDef, Iden, PostgresQueryBuilder, Table, TableCreateStatement};
use serde::{de::Visitor, Deserialize, Serialize};
use sqlx::Statement;
#[cfg(test)]
mod tests;

struct IdenString(String);

impl Iden for IdenString {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        write!(s, "{}", &self.0.to_lowercase()).unwrap();
    }
}

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
}

#[derive(Debug, Deserialize, Serialize, Getters)]
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

#[derive(Debug, Serialize)]
pub struct Schema(Vec<Field>);

impl Schema {
    pub fn inner(&self) -> &Vec<Field> {
        &self.0
    }
    pub fn table_create_statement<'a>(&self, table_name: &'a str) -> TableCreateStatement {
        let mut statement = Table::create();
        statement.table(iden_str!(table_name)).if_not_exists();

        for entry in self.inner().iter() {
            let mut column = ColumnDef::new(iden_str!(entry.name()));

            entry.nullable().then(|| column.null());

            let col_type = match entry.field_type() {
                Type::Integer => column.integer(),
                Type::Float => column.float(),
                Type::Text => column.text(),
            };
            statement.col(col_type);
        }

        let mut index = ColumnDef::new(iden_str!("id"));
        index.integer().not_null().auto_increment();

        statement.col(index.primary_key()).to_owned()
    }
}

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(SchemaVisitor)
    }
}

struct SchemaVisitor;

impl<'de> Visitor<'de> for SchemaVisitor {
    type Value = Schema;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an valid Field declaration.")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut existing = HashSet::<String>::new();
        let mut schema = Schema(vec![]);

        while let Ok(Some(entry)) = seq.next_element::<Field>() {
            if existing.contains(&entry.name) {
                Err(serde::de::Error::duplicate_field("Duplicate Field"))?;
            };
            existing.insert(entry.name.clone());
            schema.0.push(entry);
        }

        Ok(schema)
    }
}
