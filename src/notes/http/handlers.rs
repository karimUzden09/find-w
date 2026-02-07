use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
};

use super::dto::{CreateNoteRequest, NoteDto, NotesQuery};

#[utoipa::path(
    post,
    path = "/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created", body = NoteDto),
        (status = 400, description = "Invalid note payload", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Notes"
)]
pub async fn create_note(
    user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateNoteRequest>,
) -> ApiResult<(StatusCode, Json<NoteDto>)> {
    if request.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title is required".to_string()));
    }
    if request.body.trim().is_empty() {
        return Err(ApiError::BadRequest("body is required".to_string()));
    }

    let note = crate::notes::repo::create_note(&state.db, user.id, request.title, request.body)
        .await
        .map_err(ApiError::Db)?;

    Ok((
        StatusCode::CREATED,
        Json(NoteDto {
            id: note.id,
            title: note.title,
            body: note.body,
            created_at: note.created_at,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/notes",
    params(NotesQuery),
    responses(
        (status = 200, description = "User notes", body = [NoteDto]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Notes"
)]
pub async fn list_notes(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<NotesQuery>,
) -> ApiResult<(StatusCode, Json<Vec<NoteDto>>)> {
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);

    let rows = crate::notes::repo::list_notes(&state.db, user.id, limit, offset)
        .await
        .map_err(ApiError::Db)?;

    let notes = rows
        .into_iter()
        .map(|r| NoteDto {
            id: r.id,
            title: r.title,
            body: r.body,
            created_at: r.created_at,
        })
        .collect();

    Ok((StatusCode::OK, Json(notes)))
}

#[utoipa::path(
    get,
    path = "/notes/{id}",
    params(
        ("id" = Uuid, Path, description = "Note id")
    ),
    responses(
        (status = 200, description = "Note", body = NoteDto),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 404, description = "Note not found", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Notes"
)]
pub async fn get_note(
    user: AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<NoteDto>)> {
    let note = crate::notes::repo::get_note_owned(&state.db, user.id, note_id)
        .await
        .map_err(ApiError::Db)?;
    let note = note.ok_or(ApiError::NotFound)?;
    Ok((
        StatusCode::OK,
        Json(NoteDto {
            id: note.id,
            title: note.title,
            body: note.body,
            created_at: note.created_at,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/notes/{id}",
    params(
        ("id" = Uuid, Path, description = "Note id")
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 404, description = "Note not found", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Notes"
)]
pub async fn delete_note(
    user: AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let deleted = crate::notes::repo::delete_note_owned(&state.db, user.id, note_id)
        .await
        .map_err(ApiError::Db)?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT) // 204
}
