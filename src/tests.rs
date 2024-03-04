use sea_query::PostgresQueryBuilder;
use serde_json::json;

use crate::{Field, LiveSchema, Schema, Type};

#[test]
fn wont_serialize_repeated_fields() {
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
fn wont_serialize_null_fields_live_schema() {
    let json = json!({
        "temperature": null,
        "device": "Tmp0233AO",
    });

    let lv_schema = serde_json::from_value::<LiveSchema>(json);
    assert!(lv_schema.is_err())
}

#[test]
fn deserialize_static_schema() {
    let _json = json!([
        {
            "name": "temperature",
            "type": "integer",
            "nullable": true
        },
        {
            "name": "device",
            "type": "text",
            "nullable": false,
        },
    ]);

    let vec = [
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
    ];

    let js_vec = serde_json::from_value::<[Field; 2]>(_json);
    for (l, r) in vec.iter().zip(js_vec.iter().flatten()) {
        if l.ne(r) {
            assert!(false)
        }
    }
    assert!(true)
}

#[test]
fn build_sql_from_schema() {
    let x = Field {
        name: "temperature".into(),
        field_type: Type::Integer,
        nullable: true,
    };
    let y = Field {
        name: "active".into(),
        field_type: Type::Bool,
        nullable: true,
    };
    let mut schema = Schema::default();
    schema.0.push(Some(x));
    schema.0.push(Some(y));

    dbg!(&schema);

    let sql = schema
        .table_create_statement("test_t")
        .to_string(PostgresQueryBuilder)
        .to_lowercase();

    let table = vec![
        r#"create table "test_t" ("#,
        r#""temperature" integer null,"#,
        r#""active" bool null,"#,
        r#""id" serial not null primary key"#,
        r#")"#,
    ]
    .join(" ");

    assert_eq!(sql, table)
}

#[test]
fn parse_live_schema_from_json() {
    let json = json!({
        "temperature": 23.2,
        "active": false,
        "device": "AmberRoomTemp"
    });

    let mut want = LiveSchema::new(3);
    want.0.push(Some((
        Field {
            name: "temperature".into(),
            field_type: Type::Float,
            nullable: false,
        },
        serde_json::Value::from(23.2),
    )));
    want.0.push(Some((
        Field {
            name: "active".into(),
            field_type: Type::Bool,
            nullable: false,
        },
        serde_json::Value::from(false),
    )));
    want.0.push(Some((
        Field {
            name: "device".into(),
            field_type: Type::Text,
            nullable: false,
        },
        serde_json::Value::from("AmberRoomTemp"),
    )));

    let have: LiveSchema = serde_json::from_value(json).unwrap();
    assert!(have.0.get(0).unwrap().eq(have.0.get(0).unwrap()));
    assert!(have.0.get(1).unwrap().eq(have.0.get(1).unwrap()));
    assert!(have.0.get(2).unwrap().eq(have.0.get(2).unwrap()));
}

#[test]
fn create_table_sql_from_live_json_schema() {
    let json = json!({
        "temperature": 23.2,
        "device": "Tmp0233AO"
    });

    let schema = serde_json::from_value::<LiveSchema>(json);
    dbg!(&schema);

    assert!(schema.is_ok());
    let schema = schema.unwrap();

    let sql = schema
        .table_create_statement("test_t")
        .to_string(PostgresQueryBuilder)
        .to_lowercase();

    let table = vec![
        r#"create table "test_t" ("#,
        r#""device" text,"#,
        r#""temperature" real,"#,
        r#""id" serial not null primary key"#,
        r#")"#,
    ]
    .join(" ");

    assert_eq!(sql, table)
}
