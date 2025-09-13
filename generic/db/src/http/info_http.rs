use std::sync::Arc;
use crate::dto::block_dto::BlockDTO;
use crate::dto::message_dto::MessageDTO;
use crate::service::read_service::ReadService;
use axum::{extract::State, Json, Router};
use axum::extract::Query;
use axum::routing::get;
use http::StatusCode;
use serde::Deserialize;
use crate::dto::connection_dto::ConnectionDTO;
use crate::utils::cors_policy::CORS;

#[derive(Clone)]
pub struct AppState {
    pub(crate) read_service: Arc<ReadService>,
}

#[derive(Deserialize)]
pub struct HashQuery {
    hash: String,
}

#[derive(Deserialize)]
pub struct OffsetQuery {
    offset: i64,
}

#[derive(Deserialize)]
pub struct OrganizationQuery {
    organization: String,
    offset: i64,
}

async fn get_by_hash(
    Query(query): Query<HashQuery>,
    State(state): State<Arc<AppState>>
) -> Result<Json<BlockDTO>, (StatusCode, Json<MessageDTO>)> {
    match state.read_service.get_by_hash(query.hash).await {
        Ok(Some(block)) => Ok(Json(block)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(MessageDTO {
                message: "Not Found".to_string(),
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageDTO {
                message: "Server Error".to_string(),
            }),
        )),
    }
}

async fn get_page(
    Query(query): Query<OffsetQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BlockDTO>>, (StatusCode, Json<MessageDTO>)> {
    match state.read_service.get_page(query.offset).await {
        Ok(blocks) => Ok(Json(blocks)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageDTO {
                message: "Server Error".to_string(),
            }),
        )),
    }
}

async fn get_by_organisation(
    Query(query): Query<OrganizationQuery>,
    state: State<Arc<AppState>>,
) -> Result<Json<Vec<BlockDTO>>, (StatusCode, Json<MessageDTO>)> {
    match state.read_service
        .get_by_organization(query.organization, query.offset)
        .await
    {
        Ok(blocks) => Ok(Json(blocks)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageDTO {
                message: "Server Error".to_string(),
            }),
        )),
    }
}

async fn get_network(state: State<Arc<AppState>>) -> Result<Json<Vec<ConnectionDTO>>, (StatusCode, Json<MessageDTO>)> {
    match state.read_service
        .get_connections()
        .await
    {
        Ok(connections) => Ok(Json(connections)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageDTO {
                message: "Server Error".to_string(),
            }),
        )),
    }
}

pub struct InfoHttp;

impl InfoHttp {
    pub fn new(state: Arc<AppState>) -> Router {
        Router::new()
            .route("/row", get(get_by_hash))
            .route("/rows", get(get_page))
            .route("/org", get(get_by_organisation))
            .route("/network", get(get_network))
            .with_state(state)
            .layer(CORS::new())
    }
}
