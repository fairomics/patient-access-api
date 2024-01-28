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

#![allow(dead_code)]

use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge,
    PkceCodeVerifier, Scope, StandardTokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::error::Error;
use url::Url;

#[derive(Deserialize, Debug, Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub token_url: String,
    pub auth_url: String,
    pub api_scopes: Vec<String>,
    pub redirect_url: String,
    pub api_base_url: String,
}

pub fn get_oauth_client(config: &AuthConfig) -> Result<BasicClient, Box<dyn Error>> {
    let client = BasicClient::new(
        ClientId::new(config.client_id.to_string()),
        None,
        AuthUrl::new(config.auth_url.to_string())?,
        Some(TokenUrl::new(config.token_url.to_string())?),
    );
    Ok(client)
}

pub async fn oauth_pkce_access_token(
    client: &BasicClient,
    pkce_verifier: PkceCodeVerifier,
    auth_code: String,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, Box<dyn Error>> {
    let token_result = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;
    Ok(token_result)
}

pub async fn oauth_pkce_auth_url(
    client: &BasicClient,
    scopes: &Vec<String>,
) -> Result<(Url, CsrfToken, PkceCodeVerifier), Box<dyn Error>> {
    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let scopes: Vec<Scope> = Vec::from_iter(scopes.iter().map(|itm| Scope::new(itm.to_string())));

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(scopes) // Set the desired scopes.
        .set_pkce_challenge(pkce_challenge) // Set the PKCE code challenge.
        .url();

    tracing::debug!("constructed authorization url: {}", auth_url);
    Ok((auth_url, csrf_token, pkce_verifier))
}
