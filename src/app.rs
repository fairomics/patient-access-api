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

#[allow(dead_code)]
use std::error::Error;

use crate::handlers::{authz, callback, userinfo, welcome};
use crate::{
    auth::get_oauth_client,
    settings::{NgrokConfig, Settings},
};
use axum::routing::get;
use axum::Router;
use ngrok::config::TunnelBuilder;
use ngrok::tunnel::HttpTunnel;
use oauth2::basic::BasicClient;
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
#[allow(unused)]
pub struct AuthAppState {
    pub oauth_client: BasicClient,
    pub request_client: Client,
    pub pkce_verifier_secret: Arc<RwLock<String>>,
    pub access_token: Arc<RwLock<String>>,
    pub callback_query: Arc<RwLock<CallbackQuery>>,
    pub csrf_state: Arc<RwLock<String>>,
    pub api_scopes: Vec<String>,
    pub api_base_url: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CallbackQuery {
    pub state: String,
    pub code: String,
}

pub async fn get_ngrok_listener(config: &NgrokConfig) -> Result<HttpTunnel, Box<dyn Error>> {
    let listener = ngrok::Session::builder()
        .authtoken(config.auth_token.clone())
        .connect()
        .await?
        .http_endpoint()
        .domain(config.tunnel_url.clone())
        .circuit_breaker(0.5)
        .compression()
        // .webhook_verification("github", secret)
        // .oauth(
        // OauthOptions::new("google").allow_email("nlaanait@gmail.com"), // .allow_email("tristanlaanait@gmail.com"),
        // )
        .listen()
        .await?;
    Ok(listener)
}

fn initialize_state(settings: &Settings, scope: &str) -> Result<AuthAppState, Box<dyn Error>> {
    // Auth Config
    let auth_config = match scope {
        "cigna_prod" => settings.cigna_prod.clone(),
        "cigna_sandbox" => settings.cigna_sandbox.clone(),
        _ => settings.cigna_sandbox.clone(),
    };
    tracing::debug!("Auth Config: {:?}", &auth_config);

    // AuthState
    let shared_state = AuthAppState {
        request_client: reqwest::Client::builder().build()?,
        oauth_client: get_oauth_client(&auth_config)?,
        api_scopes: auth_config.api_scopes,
        pkce_verifier_secret: Arc::new(RwLock::default()),
        callback_query: Arc::new(RwLock::default()),
        access_token: Arc::new(RwLock::default()),
        csrf_state: Arc::new(RwLock::default()),
        api_base_url: auth_config.api_base_url,
    };
    Ok(shared_state)
}

pub async fn app_router(settings: &Settings, scope: &str) -> Result<Router, Box<dyn Error>> {
    // build an application with routes
    let shared_state = initialize_state(&settings, scope)?;

    let app_router = Router::new()
        .route("/", get(welcome))
        .route("/authz", get(authz))
        .route("/callback", get(callback))
        .route("/userinfo", get(userinfo))
        .with_state(shared_state);
    Ok(app_router)
}
