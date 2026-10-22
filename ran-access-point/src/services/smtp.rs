use anyhow::{Context, Result};
use lettre::message::{header, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

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

pub fn build_smtp_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
        .context("Failed to create SMTP transport")?
        .credentials(creds)
        .tls(Tls::None)
        .build();
    Ok(mailer)
}

pub async fn send_campaign_email(
    mailer: &AsyncSmtpTransport<Tokio1Executor>,
    from_email: &str,
    recipient_email: &str,
    subject: &str,
    html_body: &str,
) -> Result<()> {
    let email = Message::builder()
        .from(from_email.parse().context("Invalid sender address")?)
        .to(recipient_email.parse().context("Invalid recipient address")?)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .context("Failed to build campaign email")?;

    mailer.send(email).await.context("SMTP send failed")?;

    Ok(())
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

    let email = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] Your New Ran Online Pincode")
        .header(header::ContentType::TEXT_HTML)
        .body(html_body)
        .unwrap();
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

    let email = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] RAN Online GS : Updated Password")
        .header(header::ContentType::TEXT_HTML)
        .body(html_body)
        .unwrap();
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

    let email = Message::builder()
        .from(config.smtp_email.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("[NO-REPLY] Verify Your RAN Online GS Email")
        .header(header::ContentType::TEXT_HTML)
        .body(html_body)
        .unwrap();
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
