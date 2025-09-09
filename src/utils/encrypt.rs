use crate::error::HttpError;
use aes::Aes256;
use cbc::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use rand::Rng;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};

pub async fn encrypt_file(
    file_data: Vec<u8>,
    user_public_key: &RsaPublicKey,
) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), HttpError> {
    let mut aes_key = [0u8; 32];
    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut aes_key);
    rand::thread_rng().fill(&mut iv);

    let cipher = cbc::Encryptor::<Aes256>::new(&aes_key.into(), &iv.into());
    let mut buffer = file_data.clone();
    let encrypted_data = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, file_data.len())
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .to_vec();
    let encrypted_aes_key = user_public_key
        .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, &aes_key)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    Ok((encrypted_aes_key, encrypted_data, iv.to_vec()))
}
