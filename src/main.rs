use anyhow::Result;
use axum::Router;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use medbook_core::{
    bootstrap::{self, bootstrap},
    config, db, swagger,
};
use medbook_deliveryservice::{consumers, routes};
use utoipa::openapi::InfoBuilder;

/// Migrations embedded into the binary which helps with streamlining image building process
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<()> {
    bootstrap::init_tracing();
    bootstrap::init_env();

    let routes = routes::deliveries::routes_with_openapi()
        .merge(routes::delivery_addresses::routes_with_openapi())
        .merge(routes::patients::delivery_addresses::routes_with_openapi());

    let mut openapi = routes.get_openapi().clone();
    openapi.info = InfoBuilder::new()
        .title("MedBook DeliveryService API")
        .version("1.0.0")
        .build();

    let swagger_ui = swagger::create_swagger_ui(openapi)?;

    let app = Router::new().merge(routes).merge(swagger_ui);

    tracing::info!("Running migrations...");
    let config = config::load()?;
    let migrations_count = db::run_migrations_blocking(MIGRATIONS, &config.database.url).await?;
    tracing::info!("Run {} new migrations successfully", migrations_count);

    tracing::info!("Bootstrapping...");
    bootstrap(
        "DeliveryService",
        app,
        &[(
            "delivery.order_request",
            consumers::deliveries::order_request,
        )],
    )
    .await?;
    Ok(())
}
