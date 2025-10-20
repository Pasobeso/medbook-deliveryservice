use anyhow::Context;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use medbook_core::{
    app_error::{AppError, StdResponse},
    app_state::AppState,
    middleware,
};
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    models::{CreateDeliveryAddressEntity, DeliveryAddressEntity},
    schema::delivery_addresses,
};

/// Defines all patient-facing product routes (CRUD operations + authorization).
#[deprecated]
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/patients/delivery-addresses",
        Router::new()
            .route(
                "/my-delivery-addresses",
                routing::get(get_my_delivery_addresses),
            )
            .route("/", routing::post(create_delivery_address))
            .route("/{id}", routing::patch(update_delivery_address))
            .route("/{id}", routing::delete(delete_delivery_address))
            .route_layer(axum::middleware::from_fn(
                middleware::patients_authorization,
            )),
    )
}

/// Defines routes with OpenAPI specs. Should be used over `routes()` where possible.
pub fn routes_with_openapi() -> OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new().nest(
        "/patients/deliveries",
        OpenApiRouter::new()
            .routes(utoipa_axum::routes!(get_my_delivery_addresses))
            .routes(utoipa_axum::routes!(create_delivery_address))
            .routes(utoipa_axum::routes!(update_delivery_address))
            .routes(utoipa_axum::routes!(delete_delivery_address))
            .route_layer(axum::middleware::from_fn(
                middleware::patients_authorization,
            )),
    )
}

/// Fetch all delivery addresses belonging to the authenticated patient.
#[utoipa::path(
    get,
    path = "/my-delivery-addresses",
    tags = ["Delivery Addresses"],
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Fetched patient's delivery addresses successfully", body = StdResponse<Vec<DeliveryAddressEntity>, String>)
    )
)]
async fn get_my_delivery_addresses(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_addresses: Vec<DeliveryAddressEntity> = delivery_addresses::table
        .filter(delivery_addresses::patient_id.eq(patient_id))
        .get_results(conn)
        .await
        .context("Failed to get my delivery addresses")?;

    Ok(StdResponse {
        data: Some(delivery_addresses),
        message: Some("Get my delivery addresses successfully"),
    })
}

#[derive(Deserialize, ToSchema)]
struct CreateDeliveryAddressReq {
    recipient_name: String,
    phone_number: String,
    street_address: String,
    city: String,
    state: String,
    postal_code: String,
    country: String,
}

/// Create a new delivery address for the authenticated patient.
#[utoipa::path(
    post,
    path = "/",
    tags = ["Delivery Addresses"],
    security(("bearerAuth" = [])),
    request_body = CreateDeliveryAddressReq,
    responses(
        (status = 200, description = "Created delivery address successfully", body = StdResponse<DeliveryAddressEntity, String>)
    )
)]
async fn create_delivery_address(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
    Json(body): Json<CreateDeliveryAddressReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_address: DeliveryAddressEntity = diesel::insert_into(delivery_addresses::table)
        .values(CreateDeliveryAddressEntity {
            patient_id,
            recipient_name: body.recipient_name,
            phone_number: body.phone_number,
            street_address: body.street_address,
            city: body.city,
            state: body.state,
            postal_code: body.postal_code,
            country: body.country,
            is_default: false,
        })
        .returning(DeliveryAddressEntity::as_returning())
        .get_result(conn)
        .await
        .context("Failed to create delivery address")?;

    Ok(StdResponse {
        data: Some(delivery_address),
        message: Some("Created delivery address successfully"),
    })
}

/// Update an existing delivery address belonging to the authenticated patient.
#[utoipa::path(
    patch,
    path = "/{id}",
    tags = ["Delivery Addresses"],
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Delivery address ID to update")
    ),
    request_body = CreateDeliveryAddressReq,
    responses(
        (status = 200, description = "Updated delivery address successfully", body = StdResponse<DeliveryAddressEntity, String>)
    )
)]
async fn update_delivery_address(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
    Json(body): Json<CreateDeliveryAddressReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_address: DeliveryAddressEntity = diesel::update(
        delivery_addresses::table
            .find(id)
            .filter(delivery_addresses::patient_id.eq(patient_id)),
    )
    .set(CreateDeliveryAddressEntity {
        patient_id,
        recipient_name: body.recipient_name,
        phone_number: body.phone_number,
        street_address: body.street_address,
        city: body.city,
        state: body.state,
        postal_code: body.postal_code,
        country: body.country,
        is_default: false,
    })
    .returning(DeliveryAddressEntity::as_returning())
    .get_result(conn)
    .await
    .context("Failed to create delivery address")?;

    Ok(StdResponse {
        data: Some(delivery_address),
        message: Some("Updated delivery address successfully"),
    })
}

/// Delete a delivery address belonging to the authenticated patient.
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Delivery Addresses"],
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Delivery address ID to delete")
    ),
    responses(
        (status = 200, description = "Deleted delivery address successfully", body = StdResponse<DeliveryAddressEntity, String>)
    )
)]
async fn delete_delivery_address(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery_address: DeliveryAddressEntity = diesel::delete(
        delivery_addresses::table
            .filter(delivery_addresses::id.eq(id))
            .filter(delivery_addresses::patient_id.eq(patient_id)),
    )
    .returning(DeliveryAddressEntity::as_returning())
    .get_result(conn)
    .await
    .context("Failed to delete delivery address")?;

    Ok(StdResponse {
        data: Some(delivery_address),
        message: Some("Deleted delivery address successfully"),
    })
}
