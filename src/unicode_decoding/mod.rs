pub fn decode_unicode_escape(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'n' => {
                    result.push('\n');
                    i += 2;
                }
                b't' => {
                    result.push('\t');
                    i += 2;
                }
                b'r' => {
                    result.push('\r');
                    i += 2;
                }
                b'x' => {
                    if i + 3 < bytes.len() {
                        if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 2..i + 4]) {
                            if let Ok(value) = u8::from_str_radix(hex_str, 16) {
                                result.push(value as char);
                                i += 4;
                            }
                        }
                    }
                }
                _ => {
                    result.push(bytes[i + 1] as char);
                    i += 2;
                }
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}