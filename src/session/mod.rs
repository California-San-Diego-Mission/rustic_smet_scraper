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
        let mut nonce_cookie_header_value = self.jar.cookies(&parsed_url)
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
        
        nonce_cookie_header_value = appending_cookies::append_cookies(nonce_cookie_header_value, constant_nonce_cookies);

        let response = self.client//add headers
            .post(nonce_url)
            .header("Accept", "*/*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Connection", "keep-alive")
            .header("Content-Length", "0")
            .header("Cookie", nonce_cookie_header_value)  // Add cookie header here
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
        //clear cookies
        self.jar = Arc::new(Jar::default());
        //constants for the function
        let parsed_url = "https://id.churchofjesuschrist.org".parse::<Url>()?;
        //initial ref manager request
        let referral_manager_url = "https://referralmanager.churchofjesuschrist.org";    
        // let constant_initial_cookies: Vec<_> = [
        //     "PFpreferredHomepage=COJC",
        //     "connect.sid=s%3A8DGte01BSPEQUrsudVHQEH2PASY9F2By.GMUaAI%2BVDRt2wiWQ%2FYWRxUxOi12Tqk2l%2BCXRyi7Ve2I",
        //     "AMCV_66C5485451E56AAE0A490D45%40AdobeOrg=179643557%7CMCIDTS%7C20012%7CMCMID%7C53094083298243355950451782813184053882%7CMCAAMLH-1725137158%7C9%7CMCAAMB-1729032480%7C6G1ynYcLPuiQxYZrsz_pkqfLG9yMXBpb2zX5dvJdYQJzPXImdj0y%7CMCOPTOUT-1728779663s%7CNONE%7CvVersion%7C5.5.0"
        // ].iter().map(|s| s.to_string()).collect();
        // let mut initial_cookie_header_value = self.jar.cookies(&parsed_url)
        //     .map(|cookies| cookies.to_str().unwrap_or("").to_string())
        //     .unwrap_or_else(|| "".to_string());
        // initial_cookie_header_value = appending_cookies::append_cookies(initial_cookie_header_value, constant_initial_cookies);
        let mut response = self.client
            .get(referral_manager_url)
            // .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            // .header("Accept-Language", "en-US,en;q=0.9")
            // .header("Connection", "keep-alive")
            // .header("Cookie", initial_cookie_header_value)
            // .header("Sec-Fetch-Dest", "document")
            // .header("Sec-Fetch-Mode", "navigate")
            // .header("Sec-Fetch-Site", "none")
            // .header("Sec-Fetch-User", "?1")
            // .header("Upgrade-Insecure-Requests", "1")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            // .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            // .header("sec-ch-ua-mobile", "?0")
            // .header("sec-ch-ua-platform", "\"Linux\"")
            .send()
            .await?;
        println!("Successful initial request: {}", response_status_is_ok_from_response(&response));
            //unwrap data from the request
        let mut response_body = unwrap_response_body_from_response(response).await;
        // println!("{}", response_body);
        let encoded_state_token = extract_state_token_from_html(&response_body);
        println!("{}\n", encoded_state_token);
        let encoded_bytes = encoded_state_token.as_bytes();
        self.state_token = Some(decode_unicode_escape(encoded_bytes));
        println!("{}", self.state_token.clone().unwrap());
        panic!("Break");
        
        //work on introspect request
        let mut body = json!({
            "stateToken": self.state_token
        });
        let constant_introspect_cookies: Vec<_> = [
            //add cookies here
            "PFpreferredHomepage=COJC",
            "at_check=true",
            "AMCVS_66C5485451E56AAE0A490D45%40AdobeOrg=1",
            "s_cc=true",
            "notice_behavior=implied|us",
            "gpv_Page=church%20of%20jesus%20christ%20home",
            "gpv_cURL=www.churchofjesuschrist.org%2F",
            "s_pltp=church%20of%20jesus%20christ%20home",
            "s_ppv=church%2520of%2520jesus%2520christ%2520home%2C11%2C11%2C11%2C681%2C8%2C1"
            // ""
        ].iter().map(|s| s.to_string()).collect();

        let mut introspect_cookie_header_value = self.jar.cookies(&parsed_url)
            .map(|cookies| cookies.to_str().unwrap_or("").to_string())
            .unwrap_or_else(|| "".to_string());
        //add constant cookies
        introspect_cookie_header_value = appending_cookies::append_cookies(introspect_cookie_header_value, constant_introspect_cookies);
        // println!("Introspect cookies: {}", introspect_cookie_header_value);
        //introspect request
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/introspect")
            .header("Accept", "application/ion+json; okta-version=1.0.0")
            .header("Accept-Language", "en")
            .header("Connection", "keep-alive")
            .header("Content-Type", "application/ion+json; okta-version=1.0.0")
            .header("Cookie", introspect_cookie_header_value)
            .header("Origin", "https://id.churchofjesuschrist.org")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("X-Okta-User-Agent-Extended", "okta-auth-js/7.0.1 okta-signin-widget-7.11.3")
            .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Linux\"")
            .header("sec-ch-ua-platform-version", "\"5.15.0\"")
            // --data-raw 
            // .header("Referer", "https://referralmanager.churchofjesuschrist.org")
            .json(&body)
            .send()
            .await?;
        
        if !response_status_is_ok_from_response(&response) {
            panic!("ChurchHTTPError");
        }
        //unwrap response data
        response_body = unwrap_response_body_from_response(response).await;
        let mut json_response: serde_json::Value = serde_json::from_str(&response_body)?;    
        self.state_token = Some(json_response["stateHandle"].to_string());
        
        //nonce request
        let nonce = self.get_nonce().await?;
        println!("Nonce: {}", nonce);
        self.nonce = Some(nonce.clone()); // Save the nonce to the session

        body = json!({
            "identifier": self.username,
            "nonce": nonce,  // Include nonce here
            "stateHandle": self.state_token.clone().unwrap()
        });

        // println!("Unwrapped state token: {}", self.state_token.clone().unwrap());
        //somewhere in here, it breaks. something iswrong in the headers in this request
        
        let constant_identify_cookies: Vec<_> = [
            "PFpreferredHomepage=COJC",
            "at_check=true",
            "gpv_Page=church%20of%20jesus%20christ%20home",
            "gpv_cURL=www.churchofjesuschrist.org%2F",//Do the same thing for nonce and introspect if appropriate, though I found that v3 did not include an introspect request. Consult this url for answers about how to better insert the constant cookies. They have a good model showing how to insert them into the string with the header value, as all good things should do: https://users.rust-lang.org/t/a-good-way-to-add-cookie-to-a-request-with-reqwest-library/61041/2"www.churchofjesuschrist.org%2F"),
            "s_pltp=church%20of%20jesus%20christ%20home",
            "s_ppv=church%2520of%2520jesus%2520christ%2520home%2C11%2C11%2C11%2C681%2C8%2C1",
            "AMCVS_66C5485451E56AAE0A490D45%40AdobeOrg=1",
            "s_cc=true",
        ].iter().map(|s| s.to_string()).collect();
        
        let mut identify_cookie_header_value = self.jar.cookies(&parsed_url)
            .map(|cookies| cookies.to_str().unwrap_or("").to_string())
            .unwrap_or_else(|| "".to_string());
        //add constant cookies
        identify_cookie_header_value = appending_cookies::append_cookies(identify_cookie_header_value, constant_identify_cookies);
        //identify request
        response = self.client
            .post("https://id.churchofjesuschrist.org/idp/idx/identify")
            .header("accept", "application/json; okta-version=1.0.0")
            .header("accept-language", "en")
            .header("connection", "keep-alive")
            .header("content-type", "application/json")
            .header("Cookie", identify_cookie_header_value)  // Add cookie header here
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