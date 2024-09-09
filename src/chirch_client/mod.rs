use reqwest::Client;
use serde_json::json;
use scraper::{Html, Selector};
use crate::response_handling::{unwrap_response_body_from_response, response_status_is_ok_from_response};
use crate::unicode_decoding::decode_unicode_escape;
use rpassword::read_password;
use crate::string_extraction::{extract_state_token_from_html, extract_string_between, extract_state_handle};

// async fn parse_html(html: &str) {
//     let document = Html::parse_document(html);
//     let 
// }

pub async fn login_to_ref_manager(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let referral_manager_url = "https://referralmanager.churchofjesuschrist.org";    

    let mut response = client
        .get(referral_manager_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
        .send()
        .await?;

    println!("Status: {}", response.status());
    if let Some(content_type) = response.headers().get("Content-Type") {
        println!("Content-Type: {}", content_type.to_str()?);
    }

    let response_body = unwrap_response_body_from_response(response).await;
    
    let encoded_state_token = extract_state_token_from_html(&response_body);

    let encoded_bytes = encoded_state_token.as_bytes();

    let state_token = decode_unicode_escape(encoded_bytes);

    let body = json!({
        "stateToken": state_token
    });

    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;
    
    
    if !response_status_is_ok_from_response(&response) {
        panic!("ChurchHTTPError");
    }

    let response_body = unwrap_response_body_from_response(response).await;
    let response_body_clone = String::from(response_body.clone());
    let json_response: serde_json::Value = serde_json::from_str(&response_body)?;

    // Extract the state handle directly from the JSON
    let state_handle = json_response["stateHandle"]
        .as_str()
        .ok_or("Expected 'stateHandle' field in JSON")?;
    let state_handle_clone = extract_state_handle(&response_body_clone);

    println!("Handle: {}\n\nHandle Clone:{}", state_handle, state_handle_clone);

    let body = json!({
        "stateHandle": state_handle,
        "identifier": username
    });

    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/identify")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response_status_is_ok_from_response(&response) {
        panic!("ChurchInvalidCreds");
    }

    let response_body = unwrap_response_body_from_response(response).await;
    let json_response: serde_json::Value = serde_json::from_str(&response_body)?;

    // Extract the state handle again after identification
    let state_handle = json_response["stateHandle"]
        .as_str()
        .ok_or("Expected 'stateHandle' field in JSON")?;

    println!("Password: ___");
    let password: String = read_password().expect("Failed to read password");

    let body = json!({
        "stateHandle": state_handle,
        "credentials": {
            "passcode": password
        }
    });

    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/challenge/answer")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;

    let json_response: serde_json::Value = response.json().await?;
    let href = json_response["success"]["href"]
        .as_str()
        .ok_or("Expected 'href' field in JSON")?;

    let res = client.get(href)
        .send()
        .await?;

    if !response_status_is_ok_from_response(&res) {
        panic!("ChurchHTTPError");
    }

    Ok(())
}
