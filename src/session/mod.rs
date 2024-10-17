pub mod session_data;
use reqwest::{Client, Url};
use reqwest::cookie::CookieStore;
use crate::response_handling::{unwrap_response_body_from_response, response_status_is_ok_from_response};
use crate::unicode_decoding::decode_unicode_escape;
use crate::string_extraction::{extract_state_token_from_html};
use crate::appending_cookies;
use dotenv::dotenv;
use std::fmt;
use std::fs::File;
use std::io::Read;
use reqwest::cookie::{Jar};
use std::sync::Arc;
use serde_json::json;
use crate::session::session_data::SessionData;

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
            "{}\n State: {:?} Client ID: {:?} State Token: {:?}",
            self.username,
            self.state,
            self.client_id,
            self.state_token
        )
    }
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

    pub async fn get_nonce(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let nonce_url = "https://id.churchofjesuschrist.org/api/v1/internal/device/nonce"; // Adjust this with the actual nonce URL
        
        let parsed_url = "https://id.churchofjesuschrist.org".parse::<Url>()?;
        let mut cookie_header_value = self.jar.cookies(&parsed_url)
            .map(|cookies| cookies.to_str().unwrap_or("").to_string())
            .unwrap_or_else(|| "".to_string());
        let constant_nonce_cookies: Vec<_> = [
            "notice_behavior=implied|us",
            "at_check=true",
            "AMCVS_66C5485451E56AAE0A490D45%40AdobeOrg=1",
            "s_cc=true",
            "PFpreferredHomepage=COJC",
            "s_ips=681",
            "gpv_Page=church%20of%20jesus%20christ%20home",
            "gpv_cURL=www.churchofjesuschrist.org%2F",
            "s_pltp=church%20of%20jesus%20christ%20home",
            "s_ppv=church%2520of%2520jesus%2520christ%2520home%2C11%2C11%2C11%2C681%2C8%2C1"
            ].iter().map(|s| s.to_string()).collect();
        
        cookie_header_value = appending_cookies::append_cookies(cookie_header_value, constant_nonce_cookies);

        let response = self.client//add headers
            .post(nonce_url)
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Connection", "keep-alive")
            .header("Content-Length", "0")
            .header("Cookie", cookie_header_value)  // Add cookie header here
            .header("Origin", "https://id.churchofjesuschrist.org")
            .header("Referer", "https://id.churchofjesuschrist.org/auth/services/devicefingerprint")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Linux\"")
            .header("sec-ch-ua-platform-version", "\"5.15.0\"")
            .send()
            .await?;
    
        if !response_status_is_ok_from_response(&response) {
            panic!("Issue with nonce");
        } else {
            println!("Successful nonce request");
        }
    
        let response_body = unwrap_response_body_from_response(response).await;
        let json_response: serde_json::Value = serde_json::from_str(&response_body)?;

        // Extract the nonce from the response
        let nonce = json_response["nonce"]
            .as_str()
            .ok_or("Expected 'nonce' field in JSON")?;
    
        Ok(nonce.to_string())
    }
    

    pub async fn login_to_ref_manager(&mut self) -> Result<(), Box<dyn std::error::Error>> {    
        let referral_manager_url = "https://referralmanager.churchofjesuschrist.org";    
    
        let mut response = self.client
            .get(referral_manager_url)
            //user agent header from Elder Coxson
            // .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
            //user agent header from Elder Davis
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .send()
            .await?;
    
        // println!("Status: {}", response.status());
        // if let Some(content_type) = response.headers().get("Content-Type") {
        //     println!("Content-Type: {}", content_type.to_str()?);
        // }
    
        let mut response_body = unwrap_response_body_from_response(response).await;
        
        let encoded_state_token = extract_state_token_from_html(&response_body);
    
        let encoded_bytes = encoded_state_token.as_bytes();
    
        self.state_token = Some(decode_unicode_escape(encoded_bytes));
    
        let mut body = json!({
            "stateToken": self.state_token
        });
    
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Content-Type", "application/json;okta-version=1.0.0")
            .header("Accept", "application/json")
            .header("Referer", "https://referralmanager.churchofjesuschrist.org")
            .json(&body)
            .send()
            .await?;
        
        
        if !response_status_is_ok_from_response(&response) {
            panic!("ChurchHTTPError");
        }
    
        response_body = unwrap_response_body_from_response(response).await;
        let mut json_response: serde_json::Value = serde_json::from_str(&response_body)?;
    
        // Extract the state handle directly from the JSON
        self.state_token = Some(json_response["stateHandle"].to_string());
    
        let nonce = self.get_nonce().await?;
        self.nonce = Some(nonce.clone()); // Save the nonce to the session

        body = json!({
            "identifier": self.username,
            "nonce": nonce,  // Include nonce here
            "stateHandle": self.state_token.clone().unwrap()
        });

        // println!("Unwrapped state token: {}", self.state_token.clone().unwrap());
        //somewhere in here, it breaks. something iswrong in the headers in this request
        
        let constant_identify_cookies = [
            //proximity and PFpreferredHompage separate
            ("proximity_35e56bd795e416c8c9b87ca2cdfa0003", "eyJ6aXAiOiJERUYiLCJwMnMiOiJ5NGk1NmFGOTRRdnhDc3JRT2tGbldnIiwicDJjIjoxMDAwLCJ2ZXIiOiIxIiwiZW5jIjoiQTI1NkdDTSIsImFsZyI6IlBCRVMyLUhTNTEyK0EyNTZLVyJ9.Dz3XpyNHNPQTzNHh8vJksgmT-j3BdNCTa-RmqcRyqCFZSwbZehHEQg.Ns_3V9jZGzLm5iCn.YKrbc1qOe5zclEy6KcAKbSf2hS6S5rg6m8_Vonn2Pc8YJimSruBX-WZ_tw3RtyaOndCwrx0CXh5cY4kCSWxttXqI_gKTBQ0KGZf6MEatggDB8YuKPUI1quoxKGSuZxXiX2Q4_gET8DWk2CtcvGg4vNKtssU-NYPHDShfZ9weZsHrhg.bpDM-kJXNw57s6YZi9LocQ"),
            ("PFpreferredHomepage", "COJC"),
            ("at_check", "true"),
            ("gpv_Page", "church%20of%20jesus%20christ%20home"),
            ("gpv_cURL", "www.churchofjesuschrist.org%2F"),//Do the same thing for nonce and introspect if appropriate, though I found that v3 did not include an introspect request. Consult this url for answers about how to better insert the constant cookies. They have a good model showing how to insert them into the string with the header value, as all good things should do: https://users.rust-lang.org/t/a-good-way-to-add-cookie-to-a-request-with-reqwest-library/61041/2"www.churchofjesuschrist.org%2F"),
            ("s_pltp", "church%20of%20jesus%20christ%20home"),
            ("s_ips", "681"),
            ("s_tp", "6072"),
            ("s_ppv", "church%2520of%2520jesus%2520christ%2520home%2C11%2C11%2C11%2C681%2C8%2C1"),
            ("AMCVS_66C5485451E56AAE0A490D45%40AdobeOrg", "1"),
            ("s_cc", "true"),
        ];
        
        let parsed_url = "https://id.churchofjesuschrist.org".parse::<Url>()?;
        let mut cookie_header_value = self.jar.cookies(&parsed_url)
            .map(|cookies| cookies.to_str().unwrap_or("").to_string())
            .unwrap_or_else(|| "".to_string());
        //add constant cookies
        let mut idx = 0;
        for (cookie, url) in constant_identify_cookies.iter() {
            cookie_header_value.push_str(&cookie);
            cookie_header_value.push_str("=");
            cookie_header_value.push_str(&url);
            if idx != constant_identify_cookies.len() - 1 {
                cookie_header_value.push_str("; ");
            }
            idx += 1;
        }

        println!("Cookie header value: {}", cookie_header_value);

        //identify requestion
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/identify")
            .header("accept", "application/json; okta-version=1.0.0")
            .header("accept-language", "en")
            .header("connection", "keep-alive")
            .header("content-type", "application/json")
            .header("Cookie", cookie_header_value)  // Add cookie header here
            .header("Origin", "https://id.churchofjesuschrist.org")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            //I don't know about this one below. This really might be specific to URIM. might have to take it out
            .header("X-Device-Fingerprint", "k8AuaXw61XvR8CWhJXG7OXf3QB5MxXkt|14568ce38514d244b7f3529b5c0fe1d21d12b6e500fa436475abeb4720681a2d|c278b35ddf072fc6be8fcbd88b1ee399")
            .header("X-Okta-User-Agent-Extended", "okta-auth-js/7.0.1 okta-signin-widget-7.11.3")
            .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Linux\"")
            .header("sec-ch-ua-platform-version", "\"5.15.0\"")
            .json(&body)
            // .header("accept-encoding", "gzip, deflate, br, zstd")
            // .header("content-length", "3799")
            // .header("Content-Type", "application/json; okta-version=1.0.0")
            // .header("Accept", "application/json")
            // .header("Referer", "https://referralmanager.churchofjesuschrist.org")
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