use bcrypt::BcryptResult;

pub fn encrypt_text(text: &str) -> BcryptResult<String> {
    bcrypt::hash(text, bcrypt::DEFAULT_COST)
}

pub fn verify_encrypted_text(text: &str, hash: &str) -> BcryptResult<bool> {
    bcrypt::verify(text, hash)
}
