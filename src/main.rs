pub mod session;
pub mod response_handling;
pub mod string_extraction;
pub mod unicode_decoding;
pub mod response_logging;
pub mod appending_cookies;
use crate::session::Session;

//meaningless comment for testing commits
#[tokio::main]
async fn main() {
    let mut session = Session::new();

    let url = "https://referralmanager.churchofjesuschrist.org/";
    
    if let Err(e) = session.load_from_file(url) {
        eprintln!("Error loading session: {}", e);
    } else {
        println!("Session loaded successfully!");
    }
    if let Err(e) = session.login_to_ref_manager().await {
        eprintln!("Error logging in to session: {}", e);
    } else {
        println!("Login successful!");
    }


    // if let Err(e) = session.save(url) {
    //     eprintln!("Error saving session: {}", e);
    // } else {
    //     println!("Session saved successfully!");
    // }
    
    // let client = chirch_client::build_chirch_client();
    // println!("{}", client.username);
    // // let _ = chirch_client::login_to_ref_manager("jhdavis4").await; 
}

// -----------------------------
// -----------------------------
// -----------------------------
// -----------------------------
// -----------------------------
// -----------------------------

// use std::fs::{File, OpenOptions};
// use std::io::{Read, Write};
// use std::collections::HashMap;
// use std::time::{Duration};
// use serde::{Deserialize, Serialize};
// use serde_json::json;
// use reqwest::blocking::{Client, Response};
// use reqwest::header::USER_AGENT;
// use dotenv::dotenv;
// use std::env;
// use std::error::Error;
// use regex::Regex;

// #[derive(Debug, Serialize, Deserialize)]
// struct SessionData {
//     nonce: Option<String>,
//     state: Option<String>,
//     client_id: Option<String>,
//     state_token: Option<String>,
//     bearer: Option<String>,
//     cookies: Option<HashMap<String, String>>,
// }

// #[derive(Debug)]
// struct ChurchClient {
//     client: Client,
//     username: String,
//     password: String,
//     session_data: SessionData,
// }

// impl ChurchClient {
//     fn new() -> Result<Self, Box<dyn Error>> {
//         dotenv().ok();
//         let username = env::var("CHURCH_USERNAME")?;
//         let password = env::var("CHURCH_PASSWORD")?;

//         let session_data = ChurchClient::load_session()?;
//         let client = Client::builder()
//             .cookie_store(true)
//             .timeout(Duration::from_secs(30))
//             .build()?;

//         Ok(ChurchClient {
//             client,
//             username,
//             password,
//             session_data,
//         })
//     }

//     fn load_session() -> Result<SessionData, Box<dyn Error>> {
//         let file = File::open("session.json");
//         if let Ok(mut f) = file {
//             let mut data = String::new();
//             f.read_to_string(&mut data)?;
//             let session_data: SessionData = serde_json::from_str(&data)?;
//             Ok(session_data)
//         } else {
//             let session_data = SessionData {
//                 nonce: None,
//                 state: None,
//                 client_id: None,
//                 state_token: None,
//                 bearer: None,
//                 cookies: None,
//             };
//             let mut f = File::create("session.json")?;
//             f.write_all(b"{}")?;
//             Ok(session_data)
//         }
//     }

//     fn save_session(&self) -> Result<(), Box<dyn Error>> {
//         let mut file = OpenOptions::new().write(true).open("session.json")?;
//         let json_data = serde_json::to_string_pretty(&self.session_data)?;
//         file.set_len(0)?; // clear the file
//         file.write_all(json_data.as_bytes())?;
//         Ok(())
//     }

//     fn login(&mut self) -> Result<(), Box<dyn Error>> {
//         self.client.get("https://referralmanager.churchofjesuschrist.org")
//             .header(USER_AGENT, "Mozilla/5.0...")
//             .send()?;

//         // Assuming successful request, extract the state token
//         let res = self.client.get("https://some-url.com").send()?;
//         let state_token = self.extract_state_token(res)?;

//         self.session_data.state_token = Some(state_token.clone());

//         // Perform further requests to authenticate with username/password
//         let res = self.client.post("https://id.churchofjesuschrist.org/idp/idx/identify")
//             .json(&json!({
//                 "stateHandle": state_token,
//                 "identifier": self.username
//             }))
//             .send()?;

//         if res.status().is_success() {
//             // Update state token after login
//             let state_token_new = res.json::<HashMap<String, String>>()?["stateHandle"].clone();
//             self.session_data.state_token = Some(state_token_new);

//             // Now send password with the token
//             let res = self.client.post("https://id.churchofjesuschrist.org/idp/idx/challenge/answer")
//                 .json(&json!({
//                     "stateHandle": self.session_data.state_token.as_ref().unwrap(),
//                     "credentials": { "passcode": self.password }
//                 }))
//                 .send()?;

//             if res.status().is_success() {
//                 // Set the bearer token and save session state
//                 let bearer_token = res.json::<HashMap<String, String>>()?["token"].clone();
//                 self.session_data.bearer = Some(bearer_token);
//                 self.save_session()?;
//             } else {
//                 return Err("Failed to login with password.".into());
//             }
//         } else {
//             return Err("Failed to login with username.".into());
//         }

//         Ok(())
//     }

//     fn extract_state_token(&self, response: Response) -> Result<String, Box<dyn Error>> {
//         let text = response.text()?;
    
//         // Define the regex to capture the value of "stateToken"
//         let re = Regex::new(r#""stateToken":"(.*?)""#)?;
        
//         // Use the regex to search for the stateToken
//         let captures = re.captures(&text)
//             .ok_or("Failed to extract stateToken using regex")?;
        
//         // Extract the first capture group, which contains the stateToken
//         let state_token = captures.get(1)
//             .ok_or("Failed to capture stateToken")?
//             .as_str();
    
//         Ok(state_token.to_string())
//     }

//     fn get_referral_dashboard_counts(&self) -> Result<serde_json::Value, Box<dyn Error>> {
//         let res = self.client.get("https://referralmanager.churchofjesuschrist.org/services/facebook/dashboardCounts")
//             .bearer_auth(self.session_data.bearer.as_ref().ok_or("Missing bearer token")?)
//             .send()?;

//         if res.status().is_success() {
//             Ok(res.json()?)
//         } else {
//             Err("Failed to get referral dashboard counts.".into())
//         }
//     }
// }

// fn main() {
//     let mut client = ChurchClient::new().expect("Failed to initialize ChurchClient.");
//     client.login().expect("Failed to login.");
    
//     let counts = client.get_referral_dashboard_counts().expect("Failed to get dashboard counts.");
//     println!("{:#?}", counts);
// }





