use reqwest::{Response, StatusCode};

pub async fn unwrap_response_body(response: Result<reqwest::Response, reqwest::Error>) -> String {
    response.unwrap().text().await.unwrap()
}

pub fn response_status_is_ok(response: &Result<Response, reqwest::Error>) -> bool {
    // Match on the Result to handle Ok and Err cases
    match response {
        Ok(resp) => {
            // Print the response status
            println!("Response Status: {:?}", resp.status());
            // Check if the status is OK
            resp.status() == StatusCode::OK
        }
        Err(e) => {
            // Print the error if any
            println!("Error occurred: {:?}", e);
            false
        }
    }
}