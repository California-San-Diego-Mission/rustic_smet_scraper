pub mod gpt_chirch_client;
pub mod response_handling;
pub mod string_extraction;
pub mod unicode_decoding;
pub mod response_logging;




#[tokio::main]
async fn main() {
    gpt_chirch_client::login_to_ref_manager("jhdavis4").await; 
}




