use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::{File, ReceiveFileDetails, SentFileDetails, SharedLink, User};

#[derive(Debug, Clone)]
pub struct DbClient {
    pool: Pool<Postgres>,
}

impl DbClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

pub trait UserExt {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        name: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error>;

    async fn save_user<T>(&self, name: T, email: T, password: T) -> Result<User, sqlx::Error>
    where
        T: Into<String> + Send;

    async fn update_user_name<T>(&self, user_id: Uuid, name: T) -> Result<(), sqlx::Error>
    where
        T: Into<String> + Send;

    async fn update_user_password(
        &self,
        user_id: Uuid,
        password: String,
    ) -> Result<User, sqlx::Error>;

    async fn save_user_key(&self, user_id: Uuid, public_key: String) -> Result<(), sqlx::Error>;

    async fn search_by_email(&self, user_id: Uuid, email: String)
    -> Result<Vec<User>, sqlx::Error>;

    async fn save_encrypted_file(
        &self,
        user_id: Uuid,
        file_name: String,
        file_size: i64,
        recipient_user_id: Uuid,
        password: String,
        expiration_date: DateTime<Utc>,
        encrypted_aes_key: Vec<u8>,
        encrypted_file: Vec<u8>,
        iv: Vec<u8>,
    ) -> Result<(), sqlx::Error>;

    async fn get_shared(
        &self,
        shared_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<SharedLink>, sqlx::Error>;

    async fn get_file(&self, file_id: Uuid) -> Result<Option<File>, sqlx::Error>;
    async fn get_sent_files(
        &self,
        user_id: Uuid,
        page: u32,
        limit: usize,
    ) -> Result<(Vec<SentFileDetails>, i64), sqlx::Error>;

    async fn get_receive_files(
        &self,
        user_id: Uuid,
        page: u32,
        limit: usize,
    ) -> Result<(Vec<ReceiveFileDetails>, i64), sqlx::Error>;

    async fn delete_expired_files(&self) -> Result<(), sqlx::Error>;
}
