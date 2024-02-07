use sea_query::PostgresQueryBuilder;
use serde_json::json;

use crate::{Field, Schema, Type};

#[test]
fn cant_serialize_repeated_fields() {
    let json = json!([
        {
            "name": "temperature",
            "type": "integer",
            "nullable": true
        },
        {
            "name": "temperature",
            "type": "text",
            "nullable": false,
        }
    ]);

    assert!(serde_json::from_value::<Schema>(json).is_err())
}

#[test]
fn serialize_schema() {
    let json = json!([
        {
            "name": "temperature",
            "type": "integer",
            "nullable": true
        },
        {
            "name": "device",
            "type": "text",
            "nullable": false,
        }
    ]);

    let want = Schema(vec![
        Field {
            name: "temperature".into(),
            field_type: Type::Integer,
            nullable: true,
        },
        Field {
            name: "device".into(),
            field_type: Type::Text,
            nullable: false,
        },
    ]);

    assert_eq!(serde_json::to_value(&want).unwrap(), json)
}

#[test]
fn build_sql() {
    let schema = Schema(vec![Field {
        name: "temperature".into(),
        field_type: Type::Integer,
        nullable: true,
    }]);

    let sql = schema
        .table_create_statement("test_t")
        .to_string(PostgresQueryBuilder)
        .to_lowercase();

    let table = vec![
        r#"create table if not exists "test_t" ("#,
        r#""temperature" integer null,"#,
        r#""id" serial not null primary key"#,
        r#")"#,
    ]
    .join(" ");

    assert_eq!(sql, table)
}
