extern crate native_tls;

use native_tls::TlsConnector;
use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
    let tls_connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect("www.abc.net.au:443").expect("Could not connect to abc");
    let mut connection = tls_connector.connect("abc.net.au", stream).unwrap();
    let mut buffer = String::new();
    write!(&mut connection, "GET /news/ HTTP/1.0\r\n").expect("failed to write method");
    write!(&mut connection, "Host: www.abc.net.au\r\n").expect("failed to write Host header");
    write!(&mut connection, "\r\n").expect("failed to write end of headers");
    connection
        .read_to_string(&mut buffer)
        .expect("failed to read");
    let b = get_breaking(&buffer);
    for line in &b {
        println!("{}", line);
    }
    let exit_code = if b.len() > 0 { 0 } else { 1 };
    std::process::exit(exit_code);
}

fn get_breaking<'a>(s: &'a str) -> Vec<&'a str> {
    let tokens = split_to_tokens(s);
    let breaking = select_breaking_tokens(&tokens);
    let text_tokens = select_text_tokens(&breaking);
    let result = filter_story(&text_tokens);
    return result;
}

#[test]
fn test_get_breaking() {
    let response = include_str!("abc.html");
    assert_eq!(get_breaking(response),
               vec!["Maurice Blackburn announces potential class action against CBA in relation to AUSTRAC's legal proceedings",
                    "AUSTRAC alleged CBA's intelligent deposit machines were used to launder money and possibly finance terrorism",
                    "Shareholders says shares dropped following AUSTRAC's claims"]);
}

fn split_to_tokens<'a>(s: &'a str) -> Vec<&'a str> {
    let mut v: Vec<&str> = Vec::new();
    let mut in_tag = false;
    let mut token_start = 0;
    for (i, c) in s.char_indices() {
        if !in_tag {
            if c == '<' {
                if token_start != i {
                    v.push(&s[token_start..i]);
                }
                token_start = i;
                in_tag = true;
            }
        } else {
            if c == '>' {
                if token_start != i {
                    v.push(&s[token_start..i + 1]);
                }
                token_start = i + 1;
                in_tag = false;
            }
        }
    }
    if token_start != s.len() {
        v.push(&s[token_start..])
    }
    return v;
}

#[test]
fn test_split_to_tokens() {
    let test = |actual: Vec<&str>, expected: Vec<&str>| {
        assert_eq!(actual, expected);
    };
    test(split_to_tokens(""), vec![]);
    test(split_to_tokens("<html>"), vec!["<html>"]);
    test(split_to_tokens("<html></html>"), vec!["<html>", "</html>"]);
    test(split_to_tokens("<html>Hello World</html>"),
         vec!["<html>", "Hello World", "</html>"]);
    test(split_to_tokens("Test   <html>Hello World</html>"),
         vec!["Test   ", "<html>", "Hello World", "</html>"]);
}

fn select_breaking_tokens<'a, 'b>(tokens: &'a Vec<&'b str>) -> Vec<&'b str> {
    let mut result: Vec<&str> = Vec::new();
    let mut capture = false;
    let mut nested = 0;
    for t in tokens.iter() {
        if capture {
            result.push(t);
            if t.starts_with("<li") {
                nested += 1;
            } else if t.starts_with("</li") {
                if nested == 0 {
                    capture = false;
                } else {
                    nested -= 1;
                }
            }
        } else {
            if t.starts_with("<li") && t.contains("class=\"breaking\"") {
                result.push(t);
                capture = true;
            }
        }
    }
    return result;
}

fn select_text_tokens<'a, 'b>(tokens: &'a Vec<&'b str>) -> Vec<&'b str> {
    return tokens
               .iter()
               .map(|s| *s)
               .filter(|s| !s.starts_with("<"))
               .map(|s| s.trim())
               .filter(|s| s.len() > 0)
               .collect();
}

fn filter_story<'a, 'b>(tokens: &'a Vec<&'b str>) -> Vec<&'b str> {
    return tokens
               .iter()
               .map(|s| *s)
               .filter(|s| s.to_lowercase() != "breaking news")
               .collect();
}
