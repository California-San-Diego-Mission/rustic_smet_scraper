use reqwest::Client;
use crate::response_handling::{unwrap_response_body_from_response, response_status_is_ok_from_response};
use crate::unicode_decoding::decode_unicode_escape;
use crate::string_extraction::{extract_state_token_from_html};
use dotenv::dotenv;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use reqwest::cookie::{Jar};
use std::sync::Arc;
use std::any::type_name;
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
struct SessionData {
    nonce: Option<String>,
    state: Option<String>,
    client_id: Option<String>,
    state_token: Option<String>,
    bearer: Option<String>,
    cookies: Option<HashMap<String, String>>,
}

pub struct Session {
    pub username: String,
    password: String,
    nonce: Option<String>,
    state: Option<String>,
    client_id: Option<String>,
    state_token: Option<String>,
    bearer: Option<String>,
    client: Client,
    jar: Arc<Jar>, //store the cookies here
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // You can format this in a similar way as the Python f-string
        write!(
            f,
            "{}:{}\n State: {:?} Client ID: {:?} State Token: {:?}",
            self.username,
            self.password,
            self.state,
            self.client_id,
            self.state_token
        )
    }
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

impl Session {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(jar.clone()) // Associate the cookie jar with the client
            .build()
            .unwrap();
        dotenv().ok();//load environmental variables
        Session {
            nonce: None,
            state: None,
            username: std::env::var("CHURCH_USERNAME").expect("CHURCH_USERNAME must be set"),
            password: std::env::var("CHURCH_PASSWORD").expect("CHURCH_PASSWORD must be set"),
            client_id: None,
            state_token: None,
            bearer: None,
            client,
            jar,// Store the jar in the session
        }
    }

    pub fn load_from_file(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open("session.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let session_data: SessionData = serde_json::from_str(&contents)?;

        // Update the session object with the data from the file
        self.nonce = session_data.nonce;
        self.state = session_data.state;
        self.client_id = session_data.client_id;
        self.state_token = session_data.state_token;
        self.bearer = session_data.bearer;

        // Import cookies if they exist
        if let Some(cookies) = session_data.cookies {
            let parsed_url = url.parse()?;  // Update with your domain
            for (name, value) in cookies {
                let cookie = format!("{}={}", name, value);
                println!("{}={}", name, value);
                self.jar.add_cookie_str(&cookie, &parsed_url);
            }
        }
        println!("{}", self);
        Ok(())
    }

    pub async fn login_to_ref_manager(&mut self) -> Result<(), Box<dyn std::error::Error>> {    
        let referral_manager_url = "https://referralmanager.churchofjesuschrist.org";    
    
        let mut response = self.client
            .get(referral_manager_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
            .send()
            .await?;
    
        println!("Status: {}", response.status());
        if let Some(content_type) = response.headers().get("Content-Type") {
            println!("Content-Type: {}", content_type.to_str()?);
        }
    
        let mut response_body = unwrap_response_body_from_response(response).await;
        
        let encoded_state_token = extract_state_token_from_html(&response_body);
    
        let encoded_bytes = encoded_state_token.as_bytes();
    
        self.state_token = Some(decode_unicode_escape(encoded_bytes));
    
        let mut body = json!({
            "stateToken": self.state_token
        });
    
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Referer", "https://referralmanager.churchofjesuschrist.org")
            .json(&body)
            .send()
            .await?;
        
        
        if !response_status_is_ok_from_response(&response) {
            panic!("ChurchHTTPError");
        }
    
        response_body = unwrap_response_body_from_response(response).await;
        let response_body_clone = String::from(response_body.clone());
        let mut json_response: serde_json::Value = serde_json::from_str(&response_body)?;
    
        // Extract the state handle directly from the JSON
        self.state_token = Some(json_response["stateHandle"].to_string());
    
        body = json!({
            "stateHandle": self.state_token.clone().unwrap(),
            "identifier": self.username
        });

        println!("Unwrapped state token: {}", self.state_token.clone().unwrap());
        //somewhere in here, it breaks. something iswrong in the headers in this request
    
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/identify")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Referer", "https://referralmanager.churchofjesuschrist.org")
            .json(&body)
            .send()
            .await?;
    
        if !response_status_is_ok_from_response(&response) {
            panic!("ChurchInvalidCreds");
        }
    
        // response_body = unwrap_response_body_from_response(response).await;
        // let json_response: serde_json::Value = serde_json::from_str(&response_body)?;
    
        // Extract the state handle again after identification
        // self.state_token = Some(json_response["stateHandle"]
        //     .as_str()
        //     .ok_or("Expected 'stateHandle' field in JSON"));
        
        // let body = json!({
        //     "stateHandle": self.state_token,
        //     "credentials": {
        //         "passcode": self.password
        //     }
        // });
    
        // response = client
        //     .post("https://id.churchofjesuschrist.org/idp/idx/challenge/answer")
        //     .header("Content-Type", "application/json")
        //     .header("Accept", "application/json")
        //     .header("Referer", "https://referralmanager.churchofjesuschrist.org")
        //     .json(&body)
        //     .send()
        //     .await?;
    
        // println!("Request with password sent");
    
        // let json_response: serde_json::Value = response.json().await?;
        // let href = json_response["success"]["href"]
        //     .as_str()
        //     .ok_or("Expected 'href' field in JSON")?;
    
        // let res = client.get(href)
        //     .send()
        //     .await?;
    
        // if !response_status_is_ok_from_response(&res) {
        //     panic!("ChurchHTTPError");
        // }
    
        Ok(())
    }
    

    // pub fn save(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    //     // Prepare the cookies in a HashMap
    //     let cookies_map = HashMap::new();
    //     let parsed_url = url.parse()?; // Use your real URL
    //     let cookies = self.jar.cookies(&parsed_url).unwrap(); // Get cookies for the given URL
    
    //     println!("cookies type: {:?}", type_of(cookies.clone()));

    //     let cookies = match cookies {
    //         Some(x) => x,
    //         None => "",
    //     }
    //     // for cookie in cookies {
    //     //     println!("Cookie type: {}", type_of(cookie.clone()));
    //     //     // cookies_map.insert(cookie.name().to_string(), cookie.value().to_string());
    //     // }

    //     // Prepare session data
    //     let session_data = SessionData {
    //         nonce: self.nonce.clone(),
    //         state: self.state.clone(),
    //         client_id: self.client_id.clone(),
    //         state_token: self.state_token.clone(),
    //         cookies: Some(cookies_map),
    //         bearer: self.bearer.clone(),
    //     };

    //     // Serialize session data to JSON
    //     let json_data = serde_json::to_string_pretty(&session_data)?;

    //     // Write to file
    //     let mut file = File::create("session.json")?;
    //     file.write_all(json_data.as_bytes())?;

    //     Ok(())
    // }
}


// pub fn build_chirch_client() -> ChurchClient {
//     dotenv().ok();//load environmental variables
//     // let client =
//     //load dotenv file
//     //open session.json and write in '{}' unless there is a file exists error
//     let file = OpenOptions::new().write(true).create_new(true).open("session.json");
//     match file {
//         Ok(mut file) => {
//             // If file creation succeeded, write '{}' to the file
//             file.write_all(b"{}")?;
//         }
//         Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {
//             // Handle the case where the file already exists
//             println!("Session exists");
//         }
//         Err(e) => {
//             // Handle any other errors
//             return Err(e);
//         }
//     }

//     //open session.json and load out the session data
//     //extract nonce, state, client_id, state_token, and bearer
//     // import cookies, and if they exist, put them into the client
//     //assign all collected variables into the instance of the struct and return it
//     ChurchClient {
//         client: false,
//         username: std::env::var("CHURCH_USERNAME").expect("CHURCH_USERNAME must be set"),
//         password: std::env::var("CHURCH_PASSWORD").expect("CHURCH_PASSWORD must be set"),
//         nonce: "nonce".to_string(),
//         state: "state".to_string(),
//         client_id: false,
//         state_token: "state_token".to_string(),
//         bearer: false,
//     }
// }


// pub struct ChurchClient {
//     client: bool,
//     pub username: String,
//     password: String,
//     nonce: String,
//     state: String,
//     client_id: bool,
//     state_token: String,
//     bearer: bool,
// }



// async fn parse_html(html: &str) {
//     let document = Html::parse_document(html);
//     let 
// }





