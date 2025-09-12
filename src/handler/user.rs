use std::sync::Arc;

use axum::{Extension, Json, response::IntoResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{
        FilterUserDto, NamedUpdateDto, Response, UserData, UserPasswordUpdateDto, UserResponseDto,
    },
    error::{ErrorMessage, HttpError},
    middleware::JwtAuthMiddleware,
    utils::password,
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

pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(middleware): Extension<JwtAuthMiddleware>,
    Json(body): Json<UserPasswordUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;
    let user = &middleware.user;
    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();
    let user = app_state
        .db_client
        .get_user(Some(user_id.clone()), None, None)
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let user = user.ok_or(HttpError::unauthorized(
        ErrorMessage::InvalidToken.to_string(),
    ))?;

    let password_match = password::compare(&body.old_password, &user.password)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    if !password_match {
        return Err(HttpError::bad_request(
            "Old password is incorrect".to_string(),
        ));
    }

    let hashed_password = password::hash(&body.new_password)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    app_state
        .db_client
        .update_user_password(user_id.clone(), hashed_password)
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let response = Response {
        status: "successful",
        message: "Password updated successfully".to_string(),
    };

    Ok(Json(response))
}
