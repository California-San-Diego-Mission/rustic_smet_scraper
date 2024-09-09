use reqwest::Client;
use serde_json::json;
// use crate::response_logging::{log_response_text_and_return};
use crate::string_extraction::{extract_state_handle};
use crate::response_handling::{unwrap_response_body_from_response, response_status_is_ok_from_response};
use crate::unicode_decoding::decode_unicode_escape;
use rpassword::read_password;

pub async fn login_to_ref_manager(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let refferal_manager_url = "https://referralmanager.churchofjesuschrist.org";    

    let mut response = client
        .get(refferal_manager_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
        .send()
        .await?;
    println!("Made first request");
    // Send the request and await the response
    let mut response_body = unwrap_response_body_from_response(response).await;


    let json_data_vector: Vec<&str> = response_body
        .split("\"stateToken\":\"")
        .collect::<Vec<&str>>()[1]
        .split("\",")
        .collect();
    let json_data = String::from(json_data_vector[0]);
    let encoded_bytes = json_data.as_bytes();
    let mut state_token = decode_unicode_escape(encoded_bytes);

    let mut body = json!({
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

    response_body = unwrap_response_body_from_response(response).await;
    // response_body = unwrap_response_body_from_response(response).await;
    state_token = extract_state_handle(&response_body);

    body = json!({
        "stateHandle": state_token,
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

    response_body = unwrap_response_body_from_response(response).await;
    state_token = extract_state_handle(&response_body);

    println!("Password: ___");
    let password: String = read_password().expect("Failed to read password");

    body = json!({
        "stateHandle": state_token,
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

    let initial_res = response.json::<serde_json::Value>().await?;
    let href = initial_res["success"]["href"]
        .as_str()
        .ok_or("Expected 'href' field in JSON")?;

    let res = client.get(href)
        .send()
        .await?;

    println!("Response: {:?}", res);

    if !response_status_is_ok_from_response(&res) {
        // display_response_body_and_crash_from_response(res, "ChurchHTTPError").await;
        panic!("ChurchHTTPError");
    }

    Ok(())
}
