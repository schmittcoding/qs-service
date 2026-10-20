use actix_web::{
    body::BoxBody, cookie::time::{Duration, OffsetDateTime}, dev::{Service, ServiceRequest, ServiceResponse, Transform}, http::header, Error, HttpMessage, HttpResponse
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{
    rc::Rc,
    task::{Context, Poll},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_minutes: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>, expiration_minutes: i64) -> Self {
        Self {
            secret: secret.into(),
            expiration_minutes,
        }
    }

    pub fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.secret.as_bytes())
    }

    pub fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.secret.as_bytes())
    }
}

pub struct JwtMiddleware {
    config: Rc<JwtConfig>,
}

impl JwtMiddleware {
    pub fn new(config: JwtConfig) -> Self {
        Self {
            config: Rc::new(config),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareInner {
            service: Rc::new(service),
            config: Rc::clone(&self.config),
        }))
    }
}

pub struct JwtMiddlewareInner<S> {
    service: Rc<S>,
    config: Rc<JwtConfig>,
}

impl<S> Service<ServiceRequest> for JwtMiddlewareInner<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let config = Rc::clone(&self.config);
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Clone headers and deconstruct request
            let headers = req.headers().clone();
            let (req_head, req_payload) = req.into_parts();

            let auth_header = headers.get(header::AUTHORIZATION).cloned();

            if let Some(auth_value) = auth_header {
                if let Ok(auth_str) = auth_value.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        let validation = Validation::new(Algorithm::HS256);
                        if let Ok(token_data) = decode::<Claims>(token, &config.decoding_key(), &validation) {
                            let req = ServiceRequest::from_parts(req_head, req_payload);

                            req.extensions_mut().insert(token_data.claims);
                            return service.call(req).await;
                        }
                    }
                }
            }

            let rejected_req = ServiceRequest::from_parts(req_head, req_payload);
            let res = HttpResponse::Unauthorized().body("Authorization token missing or invalid").map_into_boxed_body();
            Ok(ServiceResponse::new(rejected_req.request().clone(), res))
        })
    }
}

pub fn generate_jwt(
    username: &str,
    role: &str,
    config: &JwtConfig,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = OffsetDateTime::now_utc() + Duration::minutes(config.expiration_minutes);
    let claims = Claims {
        sub: username.to_owned(),
        role: role.to_owned(),
        exp: expiration.unix_timestamp() as usize,
    };

    encode(&Header::default(), &claims, &config.encoding_key())
}
