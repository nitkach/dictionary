use askama::Template;
use axum::{http::StatusCode, response::IntoResponse};
use log::error;

pub(crate) struct AppError {
    code: StatusCode,
    kind: ErrorKind,
}

#[derive(Debug, Template)]
#[template(path = "error.askama.html")]
pub(crate) struct ErrorTemplate {
    code: StatusCode,
    message: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("Cannot find definitions for word: '{0}'")]
    NoDefinitionsFound(String),

    #[error("There are no records found in dictionary for word: '{0}'")]
    NoEntriesFound(String),

    #[error("The requested page does not exist.")]
    PageNotFound,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub(crate) fn new(code: StatusCode, kind: ErrorKind) -> Self {
        Self { code, kind }
    }

    pub(crate) fn word_definitions_not_found(word: String) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::NoDefinitionsFound(word))
    }

    pub(crate) fn word_entries_not_found(message: String) -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::NoEntriesFound(message))
    }

    pub(crate) fn page_not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::PageNotFound)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let message = match self.kind {
            ErrorKind::Other(error) => {
                error!("{error:?}");
                "Something went wrong.".to_owned()
            }
            _ => self.kind.to_string(),
        };

        let template = ErrorTemplate { code, message };

        (code, askama_axum::into_response(&template)).into_response()
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
