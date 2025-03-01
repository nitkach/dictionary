use crate::{
    error::AppError,
    model::{AddWordForm, WordEntry},
    repository::Repository,
};
use askama_axum::{into_response, Template};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use log::{error, info};
use tower_http::services::ServeDir;

pub(crate) fn initialize_router(shared_state: Repository) -> Router {
    Router::new()
        .route("/", get(get_index))
        .route("/words", post(post_word))
        .route("/words", get(get_words))
        .route("/words/{word}", get(get_word))
        .nest_service("/static", ServeDir::new("templates"))
        .fallback(handle_404)
        .with_state(shared_state)
}

#[derive(Debug, Template)]
#[template(path = "index.askama.html")]
struct IndexTemplate {
    words: Vec<String>,
}

#[debug_handler]
async fn get_index(State(state): State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Receive request for index page");

    let words = state.get_10_random_words().await?;

    let html = IndexTemplate { words };

    Ok(into_response(&html))
}

#[derive(Debug, Template)]
#[template(path = "word.askama.html")]
struct WordTemplate {
    word: String,
    word_entries: Vec<WordEntry>,
}

#[debug_handler]
async fn get_word(
    State(state): State<Repository>,
    Path(word): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Receive request for information about word: '{word}'");
    let Some(word_entries) = state.get_word_definitions(&word).await? else {
        return Err(AppError::word_entries_not_found(word));
    };

    let html = WordTemplate { word, word_entries };

    Ok(into_response(&html))
}

#[debug_handler]
async fn post_word(
    State(state): State<Repository>,
    Form(form): Form<AddWordForm>,
) -> Result<impl IntoResponse, AppError> {
    let word = form.word;
    info!("Receive request to add definition for word: '{word}'");
    let word_definitions = state.request_word_definitions(&word).await?;
    info!("Received definition from Dictionary API for word: '{word}'");

    let word_definitions = match word_definitions {
        crate::model::ApiResponse::Success(words_entries) => words_entries,
        crate::model::ApiResponse::Missing(crate::model::MissingResponse { .. }) => {
            error!("No definitions found for word: '{word}'");
            return Err(AppError::word_definitions_not_found(word));
        }
    };

    info!("Adding definitions to database for word: '{word}'");
    state.add_word_entries(&word, word_definitions).await?;
    info!("Successfully added definitions to database for word: '{word}'");

    Ok(Redirect::to(&format!("/words/{word}")))
}

#[derive(Debug, Template)]
#[template(path = "words.askama.html")]
struct WordsTemplate {
    words: Vec<String>,
}

#[debug_handler]
async fn get_words(State(state): State<Repository>) -> Result<impl IntoResponse, AppError> {
    info!("Receive request to list all words");

    let words = state.get_all_words().await?;

    let html = WordsTemplate { words };

    Ok(into_response(&html))
}

#[debug_handler]
async fn handle_404(_: State<Repository>) -> AppError {
    info!("User tried to access non-existing page");

    AppError::page_not_found()
}
