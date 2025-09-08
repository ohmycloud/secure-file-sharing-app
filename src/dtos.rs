use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{ReceiveFileDetails, SendFileDetails, User};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct RegisterUserDto {
    #[validate(length(min = 10, message = "Name must be at least 10 characters"))]
    pub name: String,
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Invalid email")
    )]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(
        length(min = 1, message = "Confirm password is required"),
        must_match(other = "password", message = "Passwords do not match")
    )]
    #[serde(rename = "passwordConfirm")]
    pub password_confirm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct LoginUserDto {
    #[validate(
        length(min = 1, message = "Email is required"),
        email(message = "Invalid email")
    )]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct RequestQueryDto {
    #[validate(range(min = 1))]
    pub page: Option<usize>,
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub public_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user: FilterUserDto,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseDto {
    pub status: String,
    pub data: UserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSendFileDto {
    pub file_id: String,
    pub file_name: String,
    pub recipient_email: String,
    pub expiration_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSendFileListResponseDto {
    pub status: String,
    pub files: Vec<UserSendFileDto>,
    pub results: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReceiveFileDto {
    pub file_id: String,
    pub file_name: String,
    pub sender_email: String,
    pub expiration_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReceiveFileListResponseDto {
    pub status: String,
    pub files: Vec<UserReceiveFileDto>,
    pub results: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLoginResponseDto {
    pub status: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct NamedUpdateDto {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
}

impl FilterUserDto {
    pub fn filter_user(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            name: user.name.to_owned(),
            email: user.email.to_owned(),
            public_key: user.public_key.to_owned(),
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        }
    }
}

impl UserSendFileDto {
    pub fn filter_send_user_file(file_data: &SendFileDetails) -> Self {
        Self {
            file_id: file_data.file_id.to_string(),
            file_name: file_data.file_name.to_owned(),
            recipient_email: file_data.recipient_email.to_owned(),
            expiration_date: file_data.expiration_date.unwrap(),
            created_at: file_data.created_at.unwrap(),
        }
    }

    pub fn filter_send_user_files(user: &[SendFileDetails]) -> Vec<UserSendFileDto> {
        user.iter()
            .map(UserSendFileDto::filter_send_user_file)
            .collect()
    }
}

impl UserReceiveFileDto {
    pub fn filter_receive_user_file(file_data: &ReceiveFileDetails) -> Self {
        Self {
            file_id: file_data.file_id.to_string(),
            file_name: file_data.file_name.to_owned(),
            sender_email: file_data.sender_email.to_owned(),
            expiration_date: file_data.expiration_date.unwrap(),
            created_at: file_data.created_at.unwrap(),
        }
    }

    pub fn filter_receive_user_files(user: &[ReceiveFileDetails]) -> Vec<UserReceiveFileDto> {
        user.iter()
            .map(UserReceiveFileDto::filter_receive_user_file)
            .collect()
    }
}

impl UserReceiveFileDto {
    pub fn new(
        file_id: String,
        file_name: String,
        sender_email: String,
        expiration_date: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            file_id,
            file_name,
            sender_email,
            expiration_date,
            created_at,
        }
    }
}
