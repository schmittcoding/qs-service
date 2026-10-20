use anyhow::{Context, Result};
use aws_config::Region;
use aws_sdk_sesv2::config::Credentials as AwsCredentials;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message as SesMessage};
use lettre::message::{header, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Clone)]
struct SesConfigurationSet(String);

impl header::Header for SesConfigurationSet {
    fn name() -> header::HeaderName {
        header::HeaderName::new_from_ascii_str("X-SES-CONFIGURATION-SET")
    }

    fn parse(_s: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SesConfigurationSet(_s.to_string()))
    }

    fn display(&self) -> header::HeaderValue {
        header::HeaderValue::new(Self::name(), self.0.clone())
    }
}

fn ses_configuration_set_header() -> Option<SesConfigurationSet> {
    env::var("SES_CONFIGURATION_SET")
        .ok()
        .filter(|s| !s.is_empty())
        .map(SesConfigurationSet)
}

#[derive(Debug, Deserialize)]
pub struct SmtpConfig {
    pub web_domain: String,
    pub smtp_email: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
}

impl SmtpConfig {
    pub fn from_env() -> Result<Self> {
        // Optionally load from a .env file
        dotenvy::dotenv().ok();

        Ok(SmtpConfig {
            web_domain: env::var("WEB_DOMAIN").expect("PLEASE SET WEB_DOMAIN"),
            smtp_email: env::var("SMTP_EMAIL").expect("PLEASE SET SMTP_EMAIL"),
            smtp_server: env::var("SMTP_SERVER").expect("PLEASE SET SMTP_SERVER"),
            smtp_port: env::var("SMTP_PORT")
                .expect("PLEASE SET SMTP_PORT")
                .parse()
                .unwrap(),
            smtp_user: env::var("SMTP_USER").expect("PLEASE SET SMTP_USER"),
            smtp_pass: env::var("SMTP_PASS").expect("PLEASE SET SMTP_PASS"),
        })
    }
}

pub fn render_template(
    html: &str,
    subject: &str,
    variables: &HashMap<String, String>,
) -> (String, String) {
    let mut rendered_html = html.to_string();
    let mut rendered_subject = subject.to_string();
    for (key, value) in variables {
        let placeholder = format!("{{{{{}}}}}", key);
        rendered_html = rendered_html.replace(&placeholder, value);
        rendered_subject = rendered_subject.replace(&placeholder, value);
    }
    (rendered_subject, rendered_html)
}

pub fn default_preview_variables() -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("username".to_string(), "PlayerName".to_string());
    vars.insert("email".to_string(), "player@example.com".to_string());
    vars.insert("user_num".to_string(), "12345".to_string());
    vars.insert("first_name".to_string(), "Player".to_string());
    vars.insert(
        "verification_link".to_string(),
        "https://ranonlinegs.com/verify?token=sample".to_string(),
    );
    vars.insert(
        "unsubscribe_url".to_string(),
        "https://ranonlinegs.com/contact".to_string(),
    );
    vars
}

pub struct SesConfig {
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub from_email: String,
}

impl SesConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(SesConfig {
            region: env::var("SES_REGION").context("SES_REGION must be set")?,
            access_key: env::var("SES_ACCESS_KEY").context("SES_ACCESS_KEY must be set")?,
            secret_key: env::var("SES_SECRET_KEY").context("SES_SECRET_KEY must be set")?,
            from_email: env::var("SES_FROM_EMAIL").context("SES_FROM_EMAIL must be set")?,
        })
    }
}

pub async fn build_ses_client(config: &SesConfig) -> aws_sdk_sesv2::Client {
    let region = Region::new(config.region.clone());
    let ses_config = aws_sdk_sesv2::config::Builder::new()
        .region(region)
        .credentials_provider(AwsCredentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "ses",
        ))
        .build();
    aws_sdk_sesv2::Client::from_conf(ses_config)
}

pub async fn send_campaign_email_ses(
    ses_client: &aws_sdk_sesv2::Client,
    from_email: &str,
    recipient_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<()> {
    let dest = Destination::builder()
        .to_addresses(recipient_email)
        .build();

    let subject_content = Content::builder().data(subject).charset("UTF-8").build()?;
    let html_content = Content::builder().data(html_body).charset("UTF-8").build()?;
    let body = Body::builder().html(html_content).build();
    let msg = SesMessage::builder()
        .subject(subject_content)
        .body(body)
        .build();
    let email_content = EmailContent::builder().simple(msg).build();

    let mut req = ses_client
        .send_email()
        .from_email_address(from_email)
        .destination(dest)
        .content(email_content);

    if let Ok(config_set) = env::var("SES_CONFIGURATION_SET") {
        if !config_set.is_empty() {
            req = req.configuration_set_name(config_set);
        }
    }

    req.send().await.context("SES send_email failed")?;

    Ok(())
}

pub async fn send_campaign_email(
    recipient_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<()> {
    let config = SmtpConfig::from_env()?;

    let mut builder = Message::builder()
        .from(config.smtp_email.parse()?)
        .to(recipient_email.parse()?)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML);

    if let Some(config_set) = ses_configuration_set_header() {
        builder = builder.header(config_set);
    }

    let email = builder.body(html_body.to_string())?;

    let tls = Tls::None;
    let creds = Credentials::new(config.smtp_user, config.smtp_pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)?
        .credentials(creds)
        .tls(tls)
        .build();

    mailer.send(email).await.map(|_| ()).map_err(|e| e.into())
}

pub async fn send_updated_pincode(
  username: &str,
  recipient: &str,
  pincode: &str
) -> Result<bool> {
    let config = SmtpConfig::from_env().unwrap();
    let html_body = format!(
        r#"
        <html>
          <head>
            <meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
            <style>
              /* Basic global styles */
              body {{
                font-family: Arial, sans-serif;
                font-size: 14px;
                color: #333;
                margin: 0;
                padding: 20px;
                background-color: #f4f4f4; /* Light gray background outside the container */
              }}
              /* Container with a border */
              .email-container {{
                max-width: 600px;
                margin: 0 auto;
                border: 1px solid #ccc;
                border-radius: 6px; /* Slightly rounded corners */
                background-color: #ffffff; /* White background inside the container */
                padding: 20px;
              }}
              a {{
                color: #1a73e8;
                text-decoration: none;
              }}
              .button {{
                display: inline-block;
                padding: 10px 20px;
                margin-top: 10px;
                background-color: #1a73e8;
                color: #fff !important;
                text-decoration: none;
                border-radius: 4px;
              }}
            </style>
          </head>
          <body>
            <div class="email-container">
              <p>Greetings {username},</p>
              <p>As per your request, your Ran Online Pincode has been reset.</p>
              <p>Your <b>new Pincode</b> is:</p>
              <center>
                <p>
                  {new_pincode}
                </p>
              </center>
              <p>If you did not request this Pincode reset, please <b>change your account password immediately</b> and contact our support team for assistance.</p>
              <br />
              <p>Sincerely,</p>
              <p>
                The RAN Online GS Team <br />
                <a href='https://www.ranonlinegs.com'>https://www.ranonlinegs.com</a>
              </p>
              
            </div>
          </body>
        </html>
        "#,
        username = username,
        new_pincode = pincode
    );

    let mut builder = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] Your New Ran Online Pincode")
        .header(header::ContentType::TEXT_HTML);
    if let Some(config_set) = ses_configuration_set_header() {
        builder = builder.header(config_set);
    }
    let email = builder.body(html_body).unwrap();
    let tls = Tls::None;
    let creds = Credentials::new(config.smtp_user, config.smtp_pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)?
        .credentials(creds)
        .tls(tls)
        .build();
    match mailer.send(email).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e.into()),
    }
}

pub async fn send_updated_password(
    username: &str,
    recipient: &str,
    password: &str,
) -> Result<bool> {
    let config = SmtpConfig::from_env().unwrap();
    let html_body = format!(
        r#"
        <html>
          <head>
            <meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
            <style>
              /* Basic global styles */
              body {{
                font-family: Arial, sans-serif;
                font-size: 14px;
                color: #333;
                margin: 0;
                padding: 20px;
                background-color: #f4f4f4; /* Light gray background outside the container */
              }}
              /* Container with a border */
              .email-container {{
                max-width: 600px;
                margin: 0 auto;
                border: 1px solid #ccc;
                border-radius: 6px; /* Slightly rounded corners */
                background-color: #ffffff; /* White background inside the container */
                padding: 20px;
              }}
              a {{
                color: #1a73e8;
                text-decoration: none;
              }}
              .button {{
                display: inline-block;
                padding: 10px 20px;
                margin-top: 10px;
                background-color: #1a73e8;
                color: #fff !important;
                text-decoration: none;
                border-radius: 4px;
              }}
            </style>
          </head>
          <body>
            <div class="email-container">
              <p>Hello {username},</p>
              <p>We have reset your password as requested.
              This is your new password; Please save it somewhere safe :</p>
              <center><p>
                {new_pass}
              </p></center>
              <p>If you think your account has been compromised, please change password immediately.</p>
              <p>RAN Online GS Team</p>
            </div>
          </body>
        </html>
        "#,
        username = username,
        new_pass = password
    );

    let mut builder = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] RAN Online GS : Updated Password")
        .header(header::ContentType::TEXT_HTML);
    if let Some(config_set) = ses_configuration_set_header() {
        builder = builder.header(config_set);
    }
    let email = builder.body(html_body).unwrap();
    let tls = Tls::None;
    let creds = Credentials::new(config.smtp_user, config.smtp_pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)?
        .credentials(creds)
        .tls(tls)
        .build();
    match mailer.send(email).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e.into()),
    }
}

pub async fn send_verification_email(
    username: &str,
    recipient: &str,
    verification_link: &str,
) -> Result<bool> {
    let config = SmtpConfig::from_env().unwrap();
    let html_body = format!(
        r#"
        <html>
          <head>
            <meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
            <style>
              body {{
                font-family: Arial, sans-serif;
                font-size: 14px;
                color: #333;
                margin: 0;
                padding: 20px;
                background-color: #f4f4f4;
              }}
              .email-container {{
                max-width: 600px;
                margin: 0 auto;
                border: 1px solid #ccc;
                border-radius: 6px;
                background-color: #ffffff;
                padding: 20px;
              }}
              a {{
                color: #1a73e8;
                text-decoration: none;
              }}
              .button {{
                display: inline-block;
                padding: 12px 24px;
                margin-top: 10px;
                background-color: #1a73e8;
                color: #fff !important;
                text-decoration: none;
                border-radius: 4px;
                font-weight: bold;
              }}
            </style>
          </head>
          <body>
            <div class="email-container">
              <p>Hello {username},</p>
              <p>Thank you for registering with RAN Online GS. Please verify your email address by clicking the button below:</p>
              <center>
                <p>
                  <a href='{verification_link}' class='button'>Verify Email Address</a>
                </p>
              </center>
              <p>Or copy and paste this link into your browser:</p>
              <p><a href='{verification_link}'>{verification_link}</a></p>
              <p>This link will expire in 24 hours.</p>
              <p>If you did not create an account, you can safely ignore this email.</p>
              <br />
              <p>Sincerely,</p>
              <p>
                The RAN Online GS Team <br />
                <a href='https://www.ranonlinegs.com'>https://www.ranonlinegs.com</a>
              </p>
            </div>
          </body>
        </html>
        "#,
        username = username,
        verification_link = verification_link
    );

    let mut builder = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] Verify Your RAN Online GS Email")
        .header(header::ContentType::TEXT_HTML);
    if let Some(config_set) = ses_configuration_set_header() {
        builder = builder.header(config_set);
    }
    let email = builder.body(html_body).unwrap();
    let tls = Tls::None;
    let creds = Credentials::new(config.smtp_user, config.smtp_pass);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)?
        .credentials(creds)
        .tls(tls)
        .build();
    match mailer.send(email).await {
        Ok(_) => Ok(true),
        Err(e) => Err(e.into()),
    }
}
