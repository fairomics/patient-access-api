/*
 Copyright 2024 Fairomics LLC

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
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
