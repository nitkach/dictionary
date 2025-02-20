use crate::{
    error::AppError,
    model::{PostWord, Word},
    repository::Repository,
};
use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Form, Json, Router,
};

pub(crate) fn initialize_router(shared_state: Repository) -> Router {
    let router = Router::new()
        .route("/", get(get_index))
        .route("/add", post(post_word))
        .route("/remove/{word}", delete(remove_word))
        .with_state(shared_state);

    router
}

#[debug_handler]
async fn get_index(State(state): State<Repository>) -> Result<impl IntoResponse, AppError> {
    Ok(())
}

#[debug_handler]
async fn post_word(
    State(state): State<Repository>,
    Form(word): Form<Word>,
) -> Result<impl IntoResponse, AppError> {
    let word_definitions = state.get_word_definitions(&word).await?;

    let word_definitions = match word_definitions {
        crate::model::WordDefinitions::Exists(words) => words,
        crate::model::WordDefinitions::Missing { title, .. } => {
            return Err(AppError::no_word_definitions(format!(
                "{title}: \"{word}\""
            )));
        }
    };

    state.add_word_definitions(word_definitions).await?;

    Ok(StatusCode::OK)
}

#[debug_handler]
async fn remove_word(
    State(state): State<Repository>,
    Json(json): Json<PostWord>,
) -> Result<impl IntoResponse, AppError> {
    Ok(())
}
