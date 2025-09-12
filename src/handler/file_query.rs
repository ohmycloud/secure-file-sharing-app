use std::sync::Arc;

use axum::{Extension, Json, extract::Query, response::IntoResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{RequestQueryDto, UserSendFileDto, UserSendFileListResponseDto},
    error::HttpError,
    middleware::JwtAuthMiddleware,
};

pub async fn get_user_shared_files(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(middleware): Extension<JwtAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;

    let user = &middleware.user;
    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);
    let user_id = Uuid::parse_str(&user.id.to_string()).unwrap();
    let (shared_files, total_count) = app_state
        .db_client
        .get_sent_files(user_id.clone(), page as u32, limit)
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let filter_send_files = UserSendFileDto::filter_send_user_files(&shared_files);
    let response = UserSendFileListResponseDto {
        status: "successful".to_string(),
        files: filter_send_files,
        results: total_count,
    };
    Ok(Json(response))
}
