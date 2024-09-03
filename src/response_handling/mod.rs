use reqwest::{Response, StatusCode};

pub async fn unwrap_response_body_from_result(response: Result<reqwest::Response, reqwest::Error>) -> String {
    unwrap_response_body_from_response(response.unwrap()).await
}

pub async fn unwrap_response_body_from_response(response: reqwest::Response) -> String {
    response.text().await.unwrap()
}

pub fn response_status_is_ok_from_result(response: &Result<Response, reqwest::Error>) -> bool {
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

pub fn response_status_is_ok_from_response(response: &Response) -> bool {
    // Print the response status
    println!("Response Status: {:?}", response.status());
    // Check if the status is OK
    response.status() == StatusCode::OK
        
}

pub async fn display_response_body_and_crash_from_response(response: Response, crash_message: &str) {
    println!("{}", unwrap_response_body_from_response(response).await);
    // panic!("{}", crash_message);
}
