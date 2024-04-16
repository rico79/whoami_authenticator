use std::fmt::Display;

use bcrypt::BcryptResult;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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

/** JWT claims struct
 * sub = subject -> user connected email
 * iss = issuer -> company name of the auth server
 * iat = issued at -> date of the token generation
 * exp = expiration -> end date of the token
 */
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    sub: String,
    iss: String,
    iat: i64,
    exp: i64,
}

// allow us to print the claim details
impl Display for JWTClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User ID: {}\nIssuing company: {}", self.sub, self.iss)
    }
}

/** Generate an encoded JSON Web Token
 * The subject to pass in argument is for example the mail of the authenticated user
 * The token will expire after the nb of seconds passed in argument
 */
pub fn generate_encoded_jwt(
    subject: &str,
    seconds_before_expiration: i64,
    secret: String,
) -> jsonwebtoken::errors::Result<String> {
    // Generate issued at and expiration dates (X seconds after)
    let issued_at = Utc::now().timestamp();
    let expiration = issued_at + seconds_before_expiration;

    // Create token claims
    let claims = JWTClaims {
        sub: subject.to_string(),
        iss: String::from("Brouclean Softwares Authenticator"),
        iat: issued_at,
        exp: expiration,
    };

    // Encode
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/** Decode and get sub from an encoded JSON Web Token
 */
pub fn decode_jwt_subject(
    encoded_jwt: &str,
    secret: String,
) -> jsonwebtoken::errors::Result<String> {
    // Decode the encoded token
    let token_claims = decode::<JWTClaims>(
        &encoded_jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_claims.claims.sub)
}
