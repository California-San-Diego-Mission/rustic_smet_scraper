
use reqwest;
use reqwest::Client;
pub use futures_core::future::Future;
use serde_json::json;
use text_io::read;
use crate::string_extraction::{extract_success_href, extract_state_handle};
use crate::response_handling::{unwrap_response_body, response_status_is_ok, display_response_body_and_crash};
use crate::unicode_decoding::decode_unicode_escape;
// use serde_json::Value;
// struct ChurchClient {
//     client: Client,
//     username: String,
//     password: String,
//     nonce:
//     state:
//     client_id:
//     state_token: Option<String>,
//     bearer: Option<String>,
// }



pub async fn login_to_ref_manager(username: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    //will eventually need to add functionality to clear old cookies
    //making the variable "mut" so retaking
        //ownership after the modificiations is possible.
    let request_builder = client.get(test_url)
        .header::<String, String>(String::from("User-Agent"), String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36"));
    //reqwest::RequestBuilder.header<K, V>(self, key: K, value: V) -> RequestBuilder
        
    //I'm skipping over the church http error and status code thing in chirch.py
        //This site explains status codes in the http module,
        //but it does not clearly explain how it interacts with
        //RequestBuilder in such a way that I can actually see a code come out 
            //(https://docs.rs/http/latest/http/status/struct.StatusCode.html)
    let request_result = request_builder.send();
    let output = match request_result? {
        Ok(result) => result.text(), //-> impl Future<Output = Result<String, reqwest::Error>>
        Err(_)=> panic!("HTTP request failed, site information not returned"),
    };
    let church_site_body = output?.unwrap();
    let json_data_vector: Vec<&str> = church_site_body
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
    
    let mut response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
        .header("CONTENT_TYPE", "application/json")
        .header("ACCEPT", "application/json")
        .json(&body)  // Automatically sets the Content-Type to application/json
        //the above line also automatically parses the values out into key/value pairs based on my MyResponse struct
        .send()?;
        if response_status_is_ok(&response) == false {
            panic!("ChurchHTTPError");
        }
    
    let mut response_body = unwrap_response_body(response)?;
    state_token = extract_state_handle(&response_body);
    body = json!({
        "stateHandle": state_token,
        "identifier": username
    });
    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/identify")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)  // Automatically sets the Content-Type to application/json and serializes the body
        .send()
        .await?;
    if response_status_is_ok(&response) == false {
        panic!("ChurchInvalidCreds");
    }
    response_body = unwrap_response_body(response).await;
    state_token = extract_state_handle(&response_body);
    
    println!("Password: ___");
    let password: String = read!();//take io in a second

    body = json!({
        "stateHandle": state_token,
        "credentials": {
            "passcode": password
        }
    });

    response = client
        .post("https://id.churchofjesuschrist.org/idp/idx/challenge/answer")
        .header("Content-Type", "application/json")  // Set the Content-Type header
        .header("Accept", "application/json")  // Set the Accept header
        .json(&body)  // Automatically sets the Content-Type to application/json and serializes the body
        .send()  // Send the request
        .await;
    let initial_res = response?.json::<serde_json::Value>();  // Wait for the request to complete and handle any errors
    // Step 2: Extract the "href" field from the JSON
    let href = initial_res["success"]["href"]
        .as_str()
        .ok_or("Expected 'href' field in JSON")?;

    // Step 3: Make a second GET request with allow_redirects set to false
    let res = client.get(href)
        .redirect(reqwest::redirect::Policy::none()) // This disables following redirects
        .send();

    // Optionally, you can handle the response here
    println!("Response: {:?}", res);
    
    // if response_status_is_ok(&response) == false {
    //     panic!("ChurchInvalidCreds");
    // }
    // response_body = unwrap_response_body(response)?;
    // let href = String::from(extract_success_href(&response_body));
    // // let new_client = Client::builder()
    // //     .redirect(reqwest::redirect::Policy::none()) // Disable automatic redirects
    // //     .build()
    // //     .unwrap();
    // println!("{}", href);
    // response = client
    //     .get(&href)
    //     .header("Content-Type", "application/json")  // Set the Content-Type header
    //     .header("Accept", "application/json") 
    //     .send()?;
    if response_status_is_ok(&response) == false {
        // display_response_body_and_crash(response, "ChurchHTTPError").await;
        login_to_ref_manager(username)
    }
    
    Ok(())
}
