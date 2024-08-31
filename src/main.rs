pub mod chirch_client;
pub mod response_handling;
pub mod string_extraction;
pub mod unicode_decoding;




#[tokio::main]
async fn main() {
    chirch_client::login_to_ref_manager("jhdavis4").await; 
}




