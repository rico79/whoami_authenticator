use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct AppMailer {
    from: String,
    mailer: SmtpTransport,
}

impl AppMailer {
    // Create the AppMailer struct from secrets
    pub fn new(secrets: &SecretStore) -> AppMailer {
        let mail_smtp = secrets.get("MAIL_SMTP").unwrap();
        let mail_from = secrets.get("MAIL_FROM").unwrap();
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

    // Send email
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
