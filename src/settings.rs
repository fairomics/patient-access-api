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
use clap::Parser;
use std::error::Error;

use crate::auth::AuthConfig;
use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NgrokConfig {
    pub auth_token: String,
    pub tunnel_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub cigna_sandbox: AuthConfig,
    pub cigna_prod: AuthConfig,
    pub ngrok_config: NgrokConfig,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLArgs {
    #[arg(short = 'p', long = "payer-environment", default_value_t = String::from("CIGNA_SANDBOX"))]
    payer_env: String,

    #[arg(short = 'c', long = "config")]
    config_path: String,
}

pub fn load_settings() -> Result<(Settings, String), Box<dyn Error>> {
    let args = CLArgs::parse();
    let settings = load_config(&args.config_path)?;
    let payer_env = validate_payer_env(&args.payer_env);
    tracing::info!("Configuring Payer Environment to: {}", &args.payer_env);
    Ok((settings, payer_env))
}

fn load_config(config_path: &str) -> Result<Settings, Box<dyn Error>> {
    let settings = Config::builder()
        .add_source(config::File::with_name(config_path))
        .build()?
        .try_deserialize::<Settings>()?;
    Ok(settings)
}

fn validate_payer_env(payer_env: &str) -> String {
    let valid_payer_envs = vec!["cigna_sandbox", "cigna_prod"];
    let payer_env = match valid_payer_envs
        .iter()
        .any(|x| x.eq(&payer_env.to_lowercase()))
    {
        true => payer_env.to_string(),
        false => {
            tracing::warn!(
                "Passed 'payer_env' is invalid! Must be one of {:?}, \n defaulting to: CIGNA_SANDBOX",
                valid_payer_envs
            );
            "CIGNA_SANDBOX".to_string()
        }
    };
    payer_env
}
