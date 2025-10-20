use anyhow::Context;
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use medbook_core::{
    app_error::{AppError, StdResponse},
    app_state::AppState,
};
use utoipa_axum::router::OpenApiRouter;

use crate::{models::DeliveryAddressEntity, schema::delivery_addresses};

/// Defines all patient-facing product routes (CRUD operations + authorization).
#[deprecated]
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/delivery-addresses",
        Router::new().route("/{id}", routing::get(get_delivery_address)),
    )
}

/// Defines routes with OpenAPI specs. Should be used over `routes()` where possible.
pub fn routes_with_openapi() -> OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new().nest(
        "/delivery-addresses",
        OpenApiRouter::new().routes(utoipa_axum::routes!(get_delivery_address)),
    )
}

/// Fetch a specific delivery address by its ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Delivery Addresses"],
    params(
        ("id" = i32, Path, description = "Delivery address ID to fetch")
    ),
    responses(
        (status = 200, description = "Fetched delivery address successfully", body = StdResponse<DeliveryAddressEntity, String>)
    )
)]
async fn get_delivery_address(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_address: DeliveryAddressEntity = delivery_addresses::table
        .find(id)
        .get_result(conn)
        .await
        .map_err(|_| AppError::NotFound)?;

    Ok(StdResponse {
        data: Some(delivery_address),
        message: Some("Get delivery address successfully"),
    })
}
