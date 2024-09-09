pub fn extract_string_between(response_body: &str, head: &str, tail: &str) -> String {
    String::from(
        response_body
        .split(head)
        .collect::<Vec<&str>>()[1]
        .split(tail)
        .collect::<Vec<&str>>()[0]
    )
}

pub fn extract_success_href(response_body: &str) -> String {
    extract_string_between(response_body, "\"success-redirect\",\"href\":\"", "\"")
}

pub fn extract_state_handle(response_body: &str) -> String {
    extract_string_between(response_body, "\"stateHandle\":\"", "\"")
}

pub fn extract_state_token_from_html(response_body: &str) -> String {
    extract_string_between(response_body, "\"stateToken\":\"", "\"")
}