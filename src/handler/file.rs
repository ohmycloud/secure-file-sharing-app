use std::{fs, path::PathBuf, sync::Arc};

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::{DateTime, Utc};
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey},
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{FileUploadDto, Response as ResponseDto, RetrieveFileDto},
    error::HttpError,
    middleware::JwtAuthMiddleware,
    utils::{decrypt, encrypt, password},
};

pub fn file_handle() -> Router {
    Router::new()
        .route("/upload", post(upload_file))
        .route("/register", post(retrieve_file))
}

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

pub async fn retrieve_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(middleware): Extension<JwtAuthMiddleware>,
    Json(body): Json<RetrieveFileDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let user_id = Uuid::parse_str(&middleware.user.id.to_string()).unwrap();
    let shared_id = Uuid::parse_str(&body.shared_id.to_string()).unwrap();
    let shared_link = app_state
        .db_client
        .get_shared(shared_id.clone(), user_id.clone())
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let shared_link = shared_link.ok_or_else(|| {
        HttpError::bad_request(
            "The requested shared link either does not exist or has expired".to_string(),
        )
    })?;

    let match_password = password::compare(&body.password, &shared_link.password)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    if !match_password {
        return Err(HttpError::bad_request(
            "The provided password is incorrect.".to_string(),
        ));
    }

    let file_id = match shared_link.file_id {
        Some(id) => id,
        None => return Err(HttpError::bad_request("File ID not found".to_string())),
    };
    let file = app_state
        .db_client
        .get_file(file_id.clone())
        .await
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let file = file.ok_or_else(|| {
        HttpError::bad_request(
            "The requested file either does not exist or has expired".to_string(),
        )
    })?;

    let mut path = PathBuf::from("assets/private_keys");
    path.push(format!("{}.pem", user_id.clone()));

    let private_key =
        fs::read_to_string(&path).map_err(|err| HttpError::server_error(err.to_string()))?;

    let private_key_pem = RsaPrivateKey::from_pkcs1_pem(&private_key)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    let decrypted_file = decrypt::decrypt_file(
        file.encrypted_aes_key,
        file.encrypted_file,
        file.iv,
        &private_key_pem,
    )
    .await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.file_name),
        )
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(decrypted_file))
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    Ok(response)
}
