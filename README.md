# patient-access-api
A lightweight backend server with handlers to:
1. Perform oauth2 pkce authorization flow
2. Implements redirection to authenticate a user with a payer's application and access token trading.
3. Basic requests to query data for the authenticated user.

## Dependencies
- Register as a developer with a Payer (e.g. developer.cigna.com)
- Sign up for an ngrok account to get a custom subdomain name

## Usage
Populate a `config.toml`, using the following template:
```toml
[cigna_sandbox]
client_id=""
token_url=""
auth_url=""
api_scopes=["write", "read"]
redirect_url="<the callback url you registered with cigna>"

[cigna_prod]
[cigna_sandbox]
client_id=""
token_url=""
auth_url=""
api_scopes=["write", "read"]
redirect_url="<the callback url you registered with cigna>"

[ngrok_config]
auth_token=""
tunnel_url="<your custom ngrok url>"
```
Build :
```bash
cargo build --release
```
Execute to launch the server:

```bash
$ target/release/patient_access_api --config <path-to-your-config.toml>
```

If successfully launched you should see:
```
2023-06-03T16:40:51.754303Z  INFO patient_access_api::settings: Configuring Payer Environment to: <payer-environment>
2023-06-03T16:40:52.728791Z  INFO patient_access_api: Starting Server...
2023-06-03T16:40:52.728865Z  INFO patient_access_api: Serving on <ngrok-tunnel-url>
```
For help:
```
$ patient_access_api --help
```

If the server is successfully launched, visit `<ngrok-tunnel-url>`.  
To trigger a patient authentication go to: `<ngrok-tunnel-url>/authz` 