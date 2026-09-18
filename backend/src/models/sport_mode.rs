use crate::app::AppState;
use crate::errors::ServerError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn toggle_sport_mode(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<Response, ServerError> {
    let mut sport_mode_toggle_users = state.config.sport_mode_toggle_users.lock().await;
    if !sport_mode_toggle_users.contains(&user_id) {
        sport_mode_toggle_users.push(user_id);
    }
    Ok((
        StatusCode::OK,
        format!("User {} added to sport mode toggle list", user_id),
    )
        .into_response())
}
