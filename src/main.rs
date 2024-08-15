use reqwest;
use std::collections::HashMap;
// #https://www.youtube.com/watch?v=dYVJQ-KQpdc showed me how to put together the dependenceis for reqwest, serde, and tokio, to be able to do http requests 
// #openssl was required for reqwest by default, but I was unable to make it work. I found that rustls was a valid substitute, but only if I turned off default features in reqwest
// #
#[tokio::main]
async fn main() {
    let posts_url = String::from("https://jhd.neocities.org/");
    println!("{}", posts_url);
    
    let mut post_map: HashMap<&str, &str> = HashMap::new();
    post_map.insert("userId", "1");
    post_map.insert("id", "1");
    post_map.insert("title", "foo");
    post_map.insert("body", "bar");


    let client = reqwest::Client::new();
    let resp = client
        .post(posts_url) //-> RequestBuilder
        .json(&post_map) // -> RequestBuilder
        .send() // -> impl Future<Output = Result<..., ...>>
        .await // Result<Response, Error>
        .unwrap();

    dbg!(&resp);
}
