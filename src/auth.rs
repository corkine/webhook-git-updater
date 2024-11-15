use actix_web::dev::Payload;
use actix_web::dev::ServiceRequest;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::headers::authorization::Basic;
use std::future::{ready, Ready};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user = req.extensions().get::<AuthenticatedUser>().cloned();
        match user {
            Some(u) => ready(Ok(u)),
            None => ready(Err(Error::from(actix_web::error::ErrorUnauthorized(
                "Unauthorized",
            )))),
        }
    }
}

use base64::decode;

use crate::db::get_env_var;
use crate::db::HOOK_PASSWORD;
use crate::db::HOOK_USER;
use crate::exception::ApiError;

fn extract_token_auth(req: &ServiceRequest) -> Option<BasicAuth> {
    let token = req
        .query_string()
        .split('&')
        .find(|&param| param.starts_with("token="))
        .and_then(|param| param.strip_prefix("token="))?;

    let decoded = match decode(token) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let mut parts = credentials.splitn(2, ':');
    let username = parts.next()?;
    let password = parts.next();

    Some(
        Basic::new(
            username.to_string(),
            Some(password.unwrap_or("").to_string()),
        )
        .into(),
    )
}

pub async fn validator(
    req: ServiceRequest,
    credentials: Option<BasicAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(credentials) = credentials.or(extract_token_auth(&req)) {
        let user = credentials.user_id();
        let pass = credentials.password().unwrap_or("");
        if user == get_env_var(HOOK_USER) && pass == get_env_var(HOOK_PASSWORD) {
            req.extensions_mut().insert(AuthenticatedUser {
                username: user.to_string(),
            });
            return Ok::<ServiceRequest, (Error, ServiceRequest)>(req);
        } else {
            ()
        }
    }
    Err((ApiError::Unauthorized.into(), req))
}
