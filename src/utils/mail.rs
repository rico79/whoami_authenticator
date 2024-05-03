use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use shuttle_runtime::SecretStore;

#[derive(Clone, Debug)]
pub struct AppMailer {
    from: String,
    mailer: SmtpTransport,
}

impl AppMailer {
    pub fn new(secrets: &SecretStore) -> AppMailer {
        let mail_from = format!(
            "{} <{}>",
            secrets.get("APP_NAME").unwrap(),
            secrets.get("MAIL_USER_NAME").unwrap()
        );

        let mail_smtp = secrets.get("MAIL_SMTP").unwrap();

        let mail_user_name = secrets.get("MAIL_USER_NAME").unwrap();
        let mail_password = secrets.get("MAIL_PASSWORD").unwrap();
        let credentials = Credentials::new(mail_user_name, mail_password);

        let mailer = SmtpTransport::relay(&mail_smtp)
            .unwrap()
            .credentials(credentials)
            .build();

        AppMailer {
            from: mail_from,
            mailer,
        }
    }

    pub fn send_mail(
        &self,
        to: String,
        subject: String,
        body: String,
    ) -> Result<<SmtpTransport as Transport>::Ok, <SmtpTransport as Transport>::Error> {
        let mail = Message::builder()
            .from(self.from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body)
            .unwrap();

        self.mailer.send(&mail)
    }
}
