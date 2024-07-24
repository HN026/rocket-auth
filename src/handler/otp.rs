use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::error::Error;


fn create_client() -> Result<Client, Box<dyn Error>> {
    let client = Client::builder().build()?;
    Ok(client)
}

fn create_headers() -> Result<HeaderMap, Box<dyn Error>> {
    dotenv::dotenv().ok(); 
    let postmark_token = env::var("POSTMARK_API_TOKEN").expect("POSTMARK_API_TOKEN must be set");
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert("X-Postmark-Server-Token", HeaderValue::from_str(&postmark_token)?);
    Ok(headers)
}

pub fn create_request_data(from: &str, to: &str, person_email: &str, otp: &str) -> Value {
    json!({
        "From": from,
        "To": to,
        "Subject": "Hello from Postmark",
        "HtmlBody":format!("<strong>Hello {}</strong> below is your OTP: <strong>{}</strong>",person_email ,otp),
        "TextBody": "hello there",
        "MessageStream": "broadcast",

    })
}

pub async fn send_request(data: &Value) -> Result<String, Box<dyn Error>> {
    let client = create_client()?;
    let headers = create_headers()?;
    let request = client.post("https://api.postmarkapp.com/email")
        .headers(headers)
        .json(data);
    let response = request.send().await?;
    let body = response.text().await?;
    Ok(body)
}

pub async fn send_otp_via_email(email: &str, otp: &str) -> Result<(), String> {
    let from = "piyushmishra@makerstudio.io";
    let to = email;
    let data = create_request_data(from, to, email, otp);

    match send_request(&data).await {
        Ok(response) => {
            println!("Email sent successfully: {}", response);
            Ok(())
        }
        Err(error) => {
            eprintln!("Failed to send email: {:?}", error);
            Err("Failed to send email".to_string())
        }
    }
}


