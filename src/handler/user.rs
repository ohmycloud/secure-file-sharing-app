use std::sync::Arc;

use axum::{Extension, Json, response::IntoResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{FilterUserDto, NamedUpdateDto, UserData, UserResponseDto},
    error::HttpError,
    middleware::JwtAuthMiddleware,
};

pub async fn update_user_name(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(middleware): Extension<JwtAuthMiddleware>,
    Json(body): Json<NamedUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;
    let user = &middleware.user;
    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();
    let user = app_state
        .db_client
        .update_user_name(user_id.clone(), body.name)
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let filtered_user = FilterUserDto::filter_user(&user);
    let response = UserResponseDto {
        status: "successful".to_string(),
        data: UserData {
            user: filtered_user,
        },
    };

    Ok(Json(response))
}
