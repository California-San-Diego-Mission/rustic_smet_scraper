use reqwest::{Response};


pub async fn log_response_text_and_return(response: Response) -> String {
    println!("Response Status: {}", response.status());
    println!("Response Headers: {:?}", response.headers());

    let response_text = response.text().await;
    match response_text {
        Ok(text) => {
            println!("Response Body: {}", text);
            text
        },
        Err(e) => {
            println!("Failed to read response body: {:?}", e);
            String::from("")
        },
    }
}