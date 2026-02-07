use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;
mod dto;
pub(crate) mod handlers;

pub use dto::{CreateNoteRequest, NoteDto};
pub use handlers::{create_note, delete_note, get_note, list_notes};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_note).get(list_notes))
        .route("/{id}", get(get_note).delete(delete_note))
}
