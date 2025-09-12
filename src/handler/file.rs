use std::sync::Arc;

use axum::{Extension, Json, extract::Multipart, response::IntoResponse};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::{DateTime, Utc};
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{FileUploadDto, Response as ResponseDto},
    error::HttpError,
    middleware::JwtAuthMiddleware,
    utils::{encrypt, password},
};

pub async fn upload_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(middleware): Extension<JwtAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut file_size: i64 = 0;
    let mut form_data = FileUploadDto {
        recipient_email: String::new(),
        password: String::new(),
        expiration_date: String::new(),
    };
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "fileUpload" => {
                file_name = field.file_name().unwrap_or("unknow_file").to_string();
                file_data = field.bytes().await.unwrap().to_vec();
                file_size = file_data.len() as i64;
            }
            "recipient_email" => {
                form_data.recipient_email = field.text().await.unwrap();
            }
            "password" => {
                form_data.password = field.text().await.unwrap();
            }
            "expiration_date" => {
                form_data.expiration_date = field.text().await.unwrap();
            }
            _ => {}
        }
    }

    form_data
        .validate()
        .map_err(|err| HttpError::bad_request(err.to_string()))?;

    let user = app_state
        .db_client
        .get_user(None, None, Some(&form_data.recipient_email))
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let recipient_user = user.ok_or(HttpError::bad_request("Recipient user not found"))?;
    let public_key_str = match &recipient_user.public_key {
        Some(public_key) => public_key,
        None => return Err(HttpError::bad_request("Recipient user has no public key")),
    };
    let public_key_bytes = BASE64_STANDARD
        .decode(public_key_str)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let public_key = String::from_utf8(public_key_bytes)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let public_key_pem = RsaPublicKey::from_pkcs1_pem(&public_key)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let (encrypted_aes_key, encrypted_data, iv) =
        encrypt::encrypt_file(file_data, &public_key_pem).await?;
    let user_id = Uuid::parse_str(&middleware.user.id.to_string()).unwrap();
    let hash_password = password::hash(&form_data.password)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let expiration_date = DateTime::parse_from_rfc3339(&form_data.expiration_date)
        .map_err(|err| HttpError::bad_request(err.to_string()))?
        .with_timezone(&Utc);
    let recipient_user_id = Uuid::parse_str(&recipient_user.id.to_string()).unwrap();

    app_state
        .db_client
        .save_encrypted_file(
            user_id,
            file_name,
            file_size,
            recipient_user_id,
            hash_password,
            expiration_date,
            encrypted_aes_key,
            encrypted_data,
            iv,
        )
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let response = ResponseDto {
        status: "successful",
        message: "File uploaded and encrypted successfully".to_string(),
    };

    Ok(Json(response))
}
