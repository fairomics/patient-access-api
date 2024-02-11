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

#![allow(dead_code)]

use std::error::Error;

use crate::app::{AuthAppState, CallbackQuery};
use crate::auth::{oauth_pkce_access_token, oauth_pkce_auth_url};
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use oauth2::{PkceCodeVerifier, TokenResponse};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct UserInfo {
    id: String,
    parameter: Vec<PatientID>,
    resourceType: String,
}

#[derive(Deserialize, Debug)]
pub struct PatientID {
    name: String,
    valueString: String,
}

pub async fn welcome() -> String {
    format!("Welcome Page! 🤗")
}

pub async fn authz(State(state): State<AuthAppState>) -> Redirect {
    // Get redirect authorization url
    let (auth_url, csrf_token, pkce_verifier) =
        oauth_pkce_auth_url(&state.oauth_client, &state.api_scopes)
            .await
            .unwrap();
    tracing::info!("redirecting to {auth_url}");

    // Update State
    {
        let mut lock = state.pkce_verifier_secret.write().unwrap();
        *lock = pkce_verifier.secret().to_string();
    }
    {
        let mut lock = state.csrf_state.write().unwrap();
        *lock = csrf_token.secret().to_string();
    }
    Redirect::permanent(auth_url.as_str())
}

pub async fn callback(
    Query(params): Query<CallbackQuery>,
    State(state): State<AuthAppState>,
) -> String {
    // Extract state and code
    tracing::debug!("Received State {:?}", *state.callback_query.read().unwrap());
    {
        let mut lock = state.callback_query.write().unwrap();
        *lock = params.to_owned();
    }
    tracing::debug!("Callback params {:?}", params);
    tracing::debug!("Modified State {:?}", *state.callback_query.read().unwrap());

    // validate state returned by the auth server
    let recv_state = state.callback_query.read().unwrap().clone().state;
    let ref_state = state.csrf_state.read().unwrap().clone();
    let status = match recv_state == ref_state {
        true => Ok(()),
        false => {
            tracing::error!(
                "csrf state {} != returned state from the server {}",
                ref_state,
                recv_state
            );
            Err(())
        }
    };
    if status.unwrap() != () {
        return format!("🏴‍☠️🏴‍☠️🏴‍☠️ Mismatched csrf state and state from the authorization server!!! \nNot proceeding to acquiring access token");
    }

    // get access token
    let pkce_verifier = PkceCodeVerifier::new(state.pkce_verifier_secret.read().unwrap().clone());
    let auth_code = state.callback_query.read().unwrap().code.clone();
    let token_resp = oauth_pkce_access_token(&state.oauth_client, pkce_verifier, auth_code)
        .await
        .unwrap();

    {
        let mut lock = state.access_token.write().unwrap();
        *lock = token_resp.access_token().secret().to_string();
    }

    tracing::info!("Token response: {:?}", token_resp);
    format!("Access Token Retrieved Successfully 🎉🍾🥳")
}

pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    tracing::warn!("🎬🎬🎬 received signal ctrl-C => shutdown!");
}

pub async fn userinfo(State(state): State<AuthAppState>) -> String {
    let token = state.access_token.read().unwrap().clone();
    let patient_id = get_patient_id(&state.request_client, &token, &state.api_base_url).await;
    let result = get_coverage(
        &state.request_client,
        &token,
        &state.api_base_url,
        &patient_id[0],
    )
    .await;
    format!("{}", result)
}

pub async fn get_patient_id(
    client: &Client,
    access_token: &str,
    api_base_url: &str,
) -> Vec<PatientID> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );
    let url = format!("{api_base_url}/$userinfo");
    let response = client.get(url).headers(headers).send().await.unwrap();
    let patient_id = response.json::<UserInfo>().await.unwrap().parameter;
    tracing::debug!("Extracted Patient ID: {:?}", &patient_id);
    patient_id
}

pub async fn get_coverage(
    client: &Client,
    access_token: &str,
    api_base_url: &str,
    patient_id: &PatientID,
) -> String {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );
    let url = format!(
        "{}/Coverage?patient={}",
        api_base_url, patient_id.valueString
    );
    let response = client.get(url).headers(headers).send().await.unwrap();
    let result = response.text().await.unwrap();
    tracing::debug!("Extracted Coverage: {:?}", &result);
    format!("Coverage {:?}", result)
}
