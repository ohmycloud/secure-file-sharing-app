use std::sync::Arc;

use axum::{
    Extension, Json,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::cookie::Cookie;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{LoginUserDto, RegisterUserDto, Response, UserLoginResponseDto},
    error::{ErrorMessage, HttpError},
    utils::{keys, password, token},
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

pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<LoginUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;
    let user = app_state
        .db_client
        .get_user(None, None, Some(body.email.as_str()))
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let user = user.ok_or(HttpError::bad_request(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    let password_matched = password::compare(&body.password, &user.password)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    if password_matched {
        let token = token::create_token(
            &user.id.to_string(),
            &app_state.env.jwt_secret.as_bytes(),
            app_state.env.jwt_maxage,
        )
        .map_err(|err| HttpError::server_error(err.to_string()))?;

        let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
        let cookie = Cookie::build(("token", token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();
        let response = Json(UserLoginResponseDto {
            status: "successful".to_string(),
            token,
        });
        let mut headers = HeaderMap::new();

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(HttpError::bad_request(
            ErrorMessage::WrongCredentials.to_string(),
        ))
    }
}
