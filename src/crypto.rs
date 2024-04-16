use bcrypt::BcryptResult;

/** Encrypts the text passed in arg
 * Returns the String encrypted or an Error
 */
pub fn encrypt_text(text: &str) -> BcryptResult<String> {
    bcrypt::hash(text, bcrypt::DEFAULT_COST)
}

/** Verify if the string and the hash match
 * Returns a bool or an Error
 */
pub fn verify_encrypted_text(text: &str, hash: &str) -> BcryptResult<bool> {
    bcrypt::verify(text, hash)
}
