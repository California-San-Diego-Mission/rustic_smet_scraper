use reqwest::Client;
use serde_json::json;
use crate::response_handling::{unwrap_response_body_from_response, response_status_is_ok_from_response};
use crate::unicode_decoding::decode_unicode_escape;
use rpassword::read_password;

pub async fn login_to_ref_manager(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let referral_manager_url = "https://referralmanager.churchofjesuschrist.org";    

    let mut response = client
        .get(referral_manager_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
        .send()
        .await?;

    println!("reached past first client");

    println!("Status: {}", response.status());
    if let Some(content_type) = response.headers().get("Content-Type") {
        println!("Content-Type: {}", content_type.to_str()?);
    }

    let response_body = unwrap_response_body_from_response(response).await;
    println!("reached past first unwrapping");
    // println!("Response Body: {}", response_body);

    // Parse the response body as JSON
    let json_response: serde_json::Value = serde_json::from_str(&response_body)?;
    println!("reached past first serde");
    // Extract the state token directly from the JSON
    let encoded_state_token = json_response["stateToken"]
        .as_str()
        .ok_or("Expected 'stateToken' field in JSON")?;

    let encoded_bytes = encoded_state_token.as_bytes();
    let state_token = decode_unicode_escape(encoded_bytes);

    let body = json!({
        "stateToken": state_token
    });

    println!("reached past first body definition");

    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;
    
    println!("reached past first request");
    
    if !response_status_is_ok_from_response(&response) {
        panic!("ChurchHTTPError");
    }

    let response_body = unwrap_response_body_from_response(response).await;
    let json_response: serde_json::Value = serde_json::from_str(&response_body)?;

    // Extract the state handle directly from the JSON
    let state_handle = json_response["stateHandle"]
        .as_str()
        .ok_or("Expected 'stateHandle' field in JSON")?;

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

    let new_client = Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Disable automatic redirects
        .build()?;

    let res = new_client.get(href)
        .send()
        .await?;

    if !response_status_is_ok_from_response(&res) {
        panic!("ChurchHTTPError");
    }

    Ok(())
}
