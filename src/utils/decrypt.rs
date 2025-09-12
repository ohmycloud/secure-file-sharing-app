use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use crate::error::HttpError;

pub async fn decrypt_file(
    encrypted_aes_key: Vec<u8>,
    encrypted_file: Vec<u8>,
    iv: Vec<u8>,
    user_private_key: &RsaPrivateKey,
) -> Result<Vec<u8>, HttpError> {
    let aes_key = user_private_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_aes_key)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let iv = iv;
    let cipher = cbc::Decryptor::<Aes256>::new_from_slices(&aes_key, &iv)
        .map_err(|err| HttpError::server_error(err.to_string()))?;
    let mut buffer = encrypted_file.clone();

    let decrypted_data = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|err| HttpError::server_error(err.to_string()))?;

    Ok(decrypted_data.to_vec())
}
