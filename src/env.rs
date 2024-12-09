use std::env::var;

use crate::Error;

pub(crate) fn socket_path() -> crate::Result<String> {
    var(SOCKET_PATH_ENV).map_err(|_| Error::EnvVarNotSet(SOCKET_PATH_ENV))
}

pub(crate) fn auth_token() -> crate::Result<String> {
    var(AUTH_TOKEN_ENV).map_err(|_| Error::EnvVarNotSet(AUTH_TOKEN_ENV))
}

const SOCKET_PATH_ENV: &str = "AWS_GG_NUCLEUS_DOMAIN_SOCKET_FILEPATH_FOR_COMPONENT";
const AUTH_TOKEN_ENV: &str = "SVCUID";
