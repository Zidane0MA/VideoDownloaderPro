use std::slice;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(windows::core::Error),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(windows::core::Error),
    #[error("UTF-8 conversion failed: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub fn encrypt_string(data: &str) -> Result<Vec<u8>, EncryptionError> {
    let mut data_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };

    let mut encrypted_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptProtectData(
            &mut data_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut encrypted_blob,
        )
        .map_err(EncryptionError::EncryptionFailed)?;
    }

    let encrypted_data = unsafe {
        slice::from_raw_parts(encrypted_blob.pbData, encrypted_blob.cbData as usize).to_vec()
    };

    // Free the memory allocated by CryptProtectData
    unsafe {
        LocalFree(Some(HLOCAL(encrypted_blob.pbData as *mut _)));
    }

    Ok(encrypted_data)
}

pub fn decrypt_string(data: &[u8]) -> Result<String, EncryptionError> {
    let mut data_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };

    let mut decrypted_blob = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(
            &mut data_blob,
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut decrypted_blob,
        )
        .map_err(EncryptionError::DecryptionFailed)?;
    }

    let decrypted_data = unsafe {
        slice::from_raw_parts(decrypted_blob.pbData, decrypted_blob.cbData as usize).to_vec()
    };

    let result = String::from_utf8(decrypted_data)?;

    // Free the memory allocated by CryptUnprotectData
    unsafe {
        LocalFree(Some(HLOCAL(decrypted_blob.pbData as *mut _)));
    }

    Ok(result)
}
