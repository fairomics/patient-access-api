#![allow(dead_code)]

use crate::app::{AuthAppState, CallbackQuery};
use crate::auth::{oauth_pkce_access_token, oauth_pkce_auth_url};
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use oauth2::{PkceCodeVerifier, TokenResponse};

pub async fn welcome_handler() -> String {
    format!("Welcome Page! 🤗")
}

pub async fn authz_sandbox(State(state): State<AuthAppState>) -> Redirect {
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

pub async fn callback_handler(
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
