use std::{env::var, sync::OnceLock};

use crate::Error;

pub fn socket_path() -> crate::Result<&'static str> {
    SOCKET_PATH
        .get_or_init(|| var(SOCKET_PATH_ENV).ok())
        .as_deref()
        .ok_or(Error::EnvVarNotSet(SOCKET_PATH_ENV))
}

pub fn auth_token() -> crate::Result<&'static str> {
    AUTH_TOKEN
        .get_or_init(|| var(AUTH_TOKEN_ENV).ok())
        .as_deref()
        .ok_or(Error::EnvVarNotSet(AUTH_TOKEN_ENV))
}

static SOCKET_PATH: OnceLock<Option<String>> = OnceLock::new();
static AUTH_TOKEN: OnceLock<Option<String>> = OnceLock::new();
pub const SOCKET_PATH_ENV: &str = "AWS_GG_NUCLEUS_DOMAIN_SOCKET_FILEPATH_FOR_COMPONENT";
pub const AUTH_TOKEN_ENV: &str = "SVCUID";
