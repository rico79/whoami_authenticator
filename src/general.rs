use std::fmt;

use serde::Deserialize;

pub mod dashboard;
pub mod message;
pub mod navbar;
pub mod whoami;

#[derive(Debug, Deserialize)]
pub enum AuthenticatorError {
    DatabaseError,
    CryptoError,
    NotExistingUser,
    WrongCredentials,
    MissingCredentials,
    TokenCreationFailed,
    InvalidToken,
    MissingInformation,
    PasswordsDoNotMatch,
    AlreadyExistingMail,
    InvalidUserId,
    InvalidBirthday,
    UserNotFound,
    MailConfirmationFailed,
    MailNotSent,
    AppNotFound,
    AppInvalidUri,
    InvalidDate,
}

impl fmt::Display for AuthenticatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            AuthenticatorError::DatabaseError => {
                "Un problème est survenu, veuillez réessayer plus tard"
            }
            AuthenticatorError::CryptoError => {
                "Un problème est survenu, veuillez réessayer plus tard"
            }
            AuthenticatorError::NotExistingUser => "L'utilisateur est inconnu",
            AuthenticatorError::WrongCredentials => "Les données de connexion sont incorrectes",
            AuthenticatorError::MissingCredentials => {
                "Veuillez remplir votre mail et votre mot de passe"
            }
            AuthenticatorError::TokenCreationFailed => {
                "Un problème est survenu, veuillez réessayer plus tard"
            }
            AuthenticatorError::InvalidToken => "",
            AuthenticatorError::MissingInformation => "Veuillez remplir toutes les informations",
            AuthenticatorError::PasswordsDoNotMatch => "Veuillez taper le même mot de passe",
            AuthenticatorError::AlreadyExistingMail => "Un utilisateur a déjà ce mail",
            AuthenticatorError::InvalidUserId => "L'identifiant de l'utilisateur est invalide",
            AuthenticatorError::UserNotFound => "L'utilisateur est introuvable",
            AuthenticatorError::MailConfirmationFailed => "La confirmation du mail a échouée",
            AuthenticatorError::InvalidBirthday => "La date de naissance est incorrecte",
            AuthenticatorError::AppNotFound => "L'app est introuvable",
            AuthenticatorError::MailNotSent => "Mail non envoyé",
            AuthenticatorError::AppInvalidUri => "L'Url de l'application est invalide",
            AuthenticatorError::InvalidDate => "Date invalide",
        };

        write!(f, "{}", message)
    }
}
