use axum::{http::StatusCode, Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

pub async fn entry() -> anyhow::Result<()> {
    let api_router = api::router().await?;

    let app = Router::new()
        .nest("/api", api_router)
        .fallback(generic_fallback);

    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(tcp_listener, app).await?;
    Ok(())
}

mod api {
    use axum::{
        async_trait,
        extract::{FromRequestParts, Path, Query, State},
        http::{request::Parts, StatusCode},
        response::IntoResponse,
        routing::{get, post},
        Json, RequestPartsExt, Router,
    };
    use sea_orm::{
        sea_query::{PostgresQueryBuilder, SchemaStatementBuilder},
        ColumnTrait, ConnectionTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
        Statement,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_with::{serde_as, DisplayFromStr};

    use entity::prelude::Template;
    use entity::template::{self, Model as TemplateModel};
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    pub struct DataBaseState {
        pub connection: Arc<DatabaseConnection>,
    }

    pub async fn router() -> anyhow::Result<Router> {
        let conn = Database::connect("postgres://postgres:pass@0.0.0.0:54321/filter_mock").await?;

        let state = DataBaseState {
            connection: Arc::new(conn),
        };

        Ok(Router::new()
            .route("/templates", get(query_template))
            .route("/templates/:template", post(new_template))
            .with_state(state))
    }

    // Start Region --- Query Templates

    #[serde_as]
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum TemplateQuerie {
        ById {
            #[serde_as(as = "DisplayFromStr")]
            id: i32,
        },
        ByStatus {
            active: bool,
        },
        ByNameStrict {
            name_s: String,
        },
        ByNameLoose {
            name: String,
        },
    }

    #[async_trait]
    impl<S> FromRequestParts<S> for TemplateQuerie
    where
        S: Send + Sync,
    {
        type Rejection = (StatusCode, &'static str);

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let query = parts
                .extract::<Query<Self>>()
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Bad query parameters"))?;

            Ok(query.0)
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TemplateCrudError {
        #[error("No template was found with the ID of: {0}")]
        NoTemplateForId(i32),
        #[error("No template was found with the name of: {0}")]
        NoTemplateForName(String),
        #[error("An error occurred")]
        InternalServerError,
        #[error("The given template schema is ill formed")]
        BadTemplateSchema,
    }

    impl IntoResponse for TemplateCrudError {
        fn into_response(self) -> axum::response::Response {
            match self {
                TemplateCrudError::NoTemplateForId(e) => {
                    (StatusCode::BAD_REQUEST, format!("No template for id: {e}"))
                }
                TemplateCrudError::NoTemplateForName(e) => (
                    StatusCode::BAD_REQUEST,
                    format!("No template with name: {e}"),
                ),
                TemplateCrudError::BadTemplateSchema => (
                    StatusCode::BAD_REQUEST,
                    format!("The given schema is invalid"),
                ),
                TemplateCrudError::InternalServerError => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "".into())
                }
            }
            .into_response()
        }
    }

    #[axum::debug_handler]
    /// Performs Read operations on the templates (searches)
    async fn query_template(
        query: TemplateQuerie,
        State(db): State<DataBaseState>,
    ) -> Result<Json<serde_json::Value>, TemplateCrudError> {
        use TemplateQuerie as TQ;

        let connection = db.connection.clone();

        match query {
            TQ::ById { id } => {
                let template: TemplateModel = Template::find_by_id(id)
                    .one(&*connection)
                    .await
                    // TODO - Handle SeaOrm errors
                    .map_err(|_| TemplateCrudError::InternalServerError)?
                    .ok_or_else(|| TemplateCrudError::NoTemplateForId(id))?;

                let template = serde_json::to_value(template).unwrap();
                Ok(Json(template))
            }

            TQ::ByStatus { active } => {
                let template: Vec<TemplateModel> = Template::find()
                    .filter(template::Column::Available.eq(active))
                    .all(&*connection)
                    .await
                    .map_err(|_| TemplateCrudError::InternalServerError)?;

                let templates = serde_json::to_value(template).unwrap();
                Ok(Json(templates))
            }

            TQ::ByNameStrict { name_s: name } => {
                let template: TemplateModel = Template::find()
                    .filter(template::Column::Name.eq(&name))
                    .one(&*connection)
                    .await
                    // TODO - Handle SeaOrm errors
                    .map_err(|_| TemplateCrudError::InternalServerError)?
                    .ok_or_else(|| TemplateCrudError::NoTemplateForName(name))?;

                let json = serde_json::to_value(template).unwrap();
                Ok(Json(json))
            }
            TQ::ByNameLoose { name } => {
                let template: Vec<TemplateModel> = Template::find()
                    .filter(template::Column::Name.like(format!("%{name}%")))
                    .all(&*connection)
                    .await
                    .map_err(|_| TemplateCrudError::InternalServerError)?;

                let json = serde_json::to_value(template).unwrap();
                Ok(Json(json))
            }
        }
    }

    // --- Update, Delete & Insert ---

    #[axum::debug_handler]
    async fn new_template(
        State(db): State<DataBaseState>,
        Path(template): Path<String>,
        Json(payload): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, TemplateCrudError> {
        let connection = db.connection.clone();

        let template_schema: lib_templates::Schema =
            serde_json::from_value(payload).map_err(|_| TemplateCrudError::BadTemplateSchema)?;

        let schema = template_schema
            .table_create_statement(template.as_str())
            .to_string(PostgresQueryBuilder);

        let res = connection
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                schema,
            ))
            .await;

        match res {
            Ok(s) => {
                dbg!(s);
                Ok(Json(json!({"result": "success"})))
            }
            Err(e) => {
                dbg!(e);
                Err(TemplateCrudError::BadTemplateSchema)
            }
        }
    }

    // End Region --- Query Templates
}

// Start Region --- Route Fallbacks

async fn generic_fallback() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        json!({
            "reason": "The requested resource was not found."
        })
        .into(),
    )
}

// End Region --- Route Fallbacks
