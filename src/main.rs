use anyhow::Result;
use axum::Router;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use medbook_core::{
    bootstrap::{self, bootstrap},
    config, db,
};
use medbook_deliveryservice::{consumers, routes};

/// Migrations embedded into the binary which helps with streamlining image building process
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[tokio::main]
async fn main() -> Result<()> {
    bootstrap::init_tracing();
    bootstrap::init_env();

    let app = Router::new()
        .merge(routes::deliveries::routes())
        .merge(routes::delivery_addresses::routes())
        .merge(routes::patients::delivery_addresses::routes());

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
