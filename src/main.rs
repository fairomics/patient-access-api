/*
 Copyright (c) 2024 Fairomics LLC

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use ngrok::prelude::*;
use patient_access_api::{app, handlers, settings::load_settings};
use std::error::Error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize tracing
    match std::env::var("RUST_LOG") {
        Ok(_) => (),
        Err(_) => std::env::set_var("RUST_LOG", "info"),
    }
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()?)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // load configuration
    let (settings, payer_env) = load_settings()?;

    // app service
    let app_svc = app::app_router(&settings, &payer_env)
        .await?
        .into_make_service();

    // ngrok http tunnel listener
    let listener = app::get_ngrok_listener(&settings.ngrok_config).await?;

    tracing::info!("Starting Server...");
    tracing::info!("Serving on {}", listener.url());

    // listen on ngrok tunnel and serve app router
    axum::Server::builder(listener)
        .serve(app_svc)
        .with_graceful_shutdown(handlers::shutdown_signal())
        .await?;

    Ok(())
}
