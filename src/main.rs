pub mod session;
pub mod response_handling;
pub mod string_extraction;
pub mod unicode_decoding;
pub mod response_logging;
pub mod appending_cookies;
use crate::session::Session;

#[tokio::main]
async fn main() {
    let mut session = Session::new();

    let url = "https://referralmanager.churchofjesuschrist.org/";
    
    // if let Err(e) = session.load_from_file(url) {
    //     eprintln!("Error loading session: {}", e);
    // } else {
    //     println!("Session loaded successfully!");
    // }
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
