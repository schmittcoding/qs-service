use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse};
use serde_json::json;
use tracing::{error, info};

use crate::{services::jwt::JwtConfig, states::AppState};

/// WebSocket endpoint for live campaign progress updates.
///
/// Connect: `GET /admin/v1/email/campaigns/ws?token=<JWT>`
///
/// Once connected, receives JSON frames whenever the campaign worker
/// broadcasts a progress event. No client messages are processed.
#[get("/ws")]
pub async fn campaign_progress_ws(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<Arc<AppState>>,
    jwt_config: web::Data<JwtConfig>,
    query: web::Query<WsAuthQuery>,
) -> actix_web::Result<HttpResponse> {
    // Validate JWT from query param
    if let Err(e) = validate_ws_token(&query.token, &jwt_config) {
        error!("WebSocket rejected: invalid token — {:?}", e);
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"})));
    }

    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let mut rx = app_state.campaign_progress_tx.subscribe();

    actix_web::rt::spawn(async move {
        info!("Campaign progress WebSocket client connected");

        loop {
            tokio::select! {
                // Forward broadcast events to WebSocket client
                event = rx.recv() => {
                    match event {
                        Ok(progress) => {
                            match serde_json::to_string(&progress) {
                                Ok(json_str) => {
                                    if session.text(json_str).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize progress event: {:?}", e);
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            // Slow client dropped messages — acceptable
                            tracing::warn!("Campaign WS client lagged, skipped {} events", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
                // Handle client disconnect or incoming messages
                msg = msg_stream.recv() => {
                    match msg {
                        Some(Ok(actix_ws::Message::Close(_))) | None => break,
                        Some(Ok(actix_ws::Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        _ => {} // Ignore other client messages
                    }
                }
            }
        }

        info!("Campaign progress WebSocket client disconnected");
        let _ = session.close(None).await;
    });

    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

fn validate_ws_token(token: &str, jwt_config: &JwtConfig) -> anyhow::Result<()> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use crate::services::jwt::Claims;

    let key = DecodingKey::from_secret(jwt_config.secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(token, &key, &validation)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("Invalid JWT: {:?}", e))
}
