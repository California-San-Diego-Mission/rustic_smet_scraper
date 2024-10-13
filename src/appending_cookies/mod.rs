pub fn append_cookies(header_string: String, key: &str, value: &str) -> String {
    // takes ownership of a header string, appends the cookie, and returns ownership of the header string
    header_string.push_string(key);
    header_string.push_str("=");
    //need to figure out a good way to handle adding a ";" to the end of every single cookie except the last one
    // header_string.push_str()
    header_string //return ownership
}

let mut header_value = String::new();
for (idx, (k, v)) in cookies.iter().enumerate() {
    // may allocate per .push_str, not the most efficient
    header_value.push_str(&k);
    header_value.push_str("=");
    header_value.push_str(&v.replace(";", "%3B"));
    if idx != cookies.len() - 1 {
        header_value.push_str(";");
    }
}