use sea_query::PostgresQueryBuilder;
use serde_json::json;

use crate::{Field, LiveSchema, Schema, Type};

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
        r#"create table "test_t" ("#,
        r#""temperature" integer null,"#,
        r#""id" serial not null primary key"#,
        r#")"#,
    ]
    .join(" ");

    assert_eq!(sql, table)
}

#[test]
fn parse_live_schema() {
    let json = json!({
        "temperature": 23.2,
    });

    let mut want = LiveSchema::default();
    want.0.get_mut(0).unwrap().replace(Field {
        name: "temperature".into(),
        field_type: Type::Float,
        nullable: false,
    });

    let have: LiveSchema = serde_json::from_value(json).unwrap();
    assert!(have.0.get(0).unwrap().eq(have.0.get(0).unwrap()))
}
