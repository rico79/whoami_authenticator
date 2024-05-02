use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use shuttle_runtime::SecretStore;

/// App mailer
/// Structure to use when an email has to be sent
#[derive(Clone, Debug)]
pub struct AppMailer {
    from: String,
    mailer: SmtpTransport,
}

impl AppMailer {
    //// App mailer creation
    /// Create the AppMailer struct from secrets informations
    pub fn new(secrets: &SecretStore) -> AppMailer {
        let mail_smtp = secrets.get("MAIL_SMTP").unwrap();
        let mail_from = format!(
            "{} <{}>",
            secrets.get("APP_NAME").unwrap(),
            secrets.get("MAIL_USER_NAME").unwrap()
        );
        let mail_user_name = secrets.get("MAIL_USER_NAME").unwrap();
        let mail_password = secrets.get("MAIL_PASSWORD").unwrap();

        let creds = Credentials::new(mail_user_name, mail_password);

        let mailer = SmtpTransport::relay(&mail_smtp)
            .unwrap()
            .credentials(creds)
            .build();

        AppMailer {
            from: mail_from,
            mailer,
        }
    }

    /// Send email
    /// Send an email to the receiver and the mail content passed in arguments
    pub fn send(
        &self,
        to: String,
        subject: String,
        body: String,
    ) -> Result<<SmtpTransport as Transport>::Ok, <SmtpTransport as Transport>::Error> {
        // Create the email
        let email = Message::builder()
            .from(self.from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body)
            .unwrap();

        // Send email
        self.mailer.send(&email)
    }
}
