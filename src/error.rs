use axum::{http::StatusCode, response::IntoResponse};
use log::error;

pub(crate) struct AppError {
    code: StatusCode,
    kind: ErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("{0}")]
    NoDefinitionsFound(String),

    #[error("{0}")]
    NoEntriesFound(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub(crate) fn new(code: StatusCode, kind: ErrorKind) -> Self {
        Self { code, kind }
    }

    pub(crate) fn word_definitions_not_found(message: String) -> Self {
        Self {
            code: StatusCode::NOT_FOUND,
            kind: ErrorKind::NoDefinitionsFound(message),
        }
    }

    pub(crate) fn word_entries_not_found(message: String) -> Self {
        Self {
            code: StatusCode::NOT_FOUND,
            kind: ErrorKind::NoEntriesFound(message),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let body = match self.kind {
            ErrorKind::Other(error) => {
                error!("{error:?}");
                "something went wrong".to_owned()
            }
            _ => self.kind.to_string(),
        };

        (code, body).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            kind: ErrorKind::Other(err.into()),
        }
    }
}
