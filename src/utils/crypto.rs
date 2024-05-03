use crate::general::AuthenticatorError;

pub fn encrypt_text(text: &str) -> Result<String, AuthenticatorError> {
    bcrypt::hash(text, bcrypt::DEFAULT_COST).map_err(|error| {
        tracing::error!("{}", error);
        AuthenticatorError::CryptoError
    })
}

pub fn verify_encrypted_text(text: &str, hash: &str) -> Result<bool, AuthenticatorError> {
    bcrypt::verify(text, hash).map_err(|error| {
        tracing::error!("{}", error);
        AuthenticatorError::CryptoError
    })
}
