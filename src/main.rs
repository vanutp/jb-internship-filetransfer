mod headers;
mod http;

use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use crate::http::http_request;

fn main() {
    let mut data = Vec::new();
    let url = "http://127.0.0.1:8080";
    loop {
        let headers = vec![
            ("Range".to_string(), format!("bytes={}-", data.len()))
        ];
        let mut resp = http_request(url, headers.into()).unwrap();
        data.append(&mut resp.body);
        println!("{} bytes downloaded", data.len());
        if !resp.premature_eof {
            break
        }
    }

    let mut file = File::create("result.txt").unwrap();
    file.write_all(&data).unwrap();
}
