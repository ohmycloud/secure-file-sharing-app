use std::sync::Arc;

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{RegisterUserDto, Response},
    error::{ErrorMessage, HttpError},
    utils::{keys, password},
};

pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(user): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    user.validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;

    let hash_password =
        password::hash(&user.password).map_err(|err| HttpError::server_error(err.to_string()))?;

    let user = app_state
        .db_client
        .save_user(user.name, user.email, hash_password)
        .await;

    match user {
        Ok(user) => {
            let _key_result = keys::generete_key(app_state, user).await?;
            Ok((
                StatusCode::CREATED,
                Json(Response {
                    status: "successful",
                    message: "User registered successfully".to_string(),
                }),
            ))
        }
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                Err(HttpError::unique_constraint_violation(
                    ErrorMessage::EmailAlreadyExists.to_string(),
                ))
            } else {
                Err(HttpError::server_error(db_err.to_string()))
            }
        }
        Err(err) => Err(HttpError::server_error(err.to_string())),
    }
}
