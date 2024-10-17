pub fn append_cookies(mut header_value: String, constant_cookies: Vec<String>) -> String {
    //adds all cookies in the vector to header_value, then returns ownership
    //assumes that each cookie is formatted as "cookie=url" with no semicolon at the end
    let mut idx = 0;
    for cookie in constant_cookies.iter() {
        header_value.push_str(&cookie);
        if idx != constant_cookies.len() - 1 {
            header_value.push_str("; ");
        }
        idx += 1;
    }
    header_value
}

          
        
        
