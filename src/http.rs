use crate::headers::{Header, Headers};
use crate::http::RequestError::*;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

pub type StatusCode = u16;

#[derive(Debug, Clone)]
pub enum RequestError {
    InvalidUrl,
    ConnectionFailed,
    WriteError,
    ReadError,
    ResponseParseError(&'static str),
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            InvalidUrl => "The URL is invalid",
            ConnectionFailed => "Connection failed",
            WriteError => "Error writing request",
            ReadError => "Error reading response",
            ResponseParseError(msg) => &format!("Error parsing response: {}", msg),
        };
        write!(f, "{}", msg)
    }
}

pub struct HttpResponse {
    pub status_code: StatusCode,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub premature_eof: bool,
}

pub fn http_request(url: &str, mut headers: Headers) -> Result<HttpResponse, RequestError> {
    if !url.starts_with("http://") {
        return Err(InvalidUrl);
    }
    let url = url.trim_start_matches("http://");
    let (addr, path) = url.split_at(url.find('/').unwrap_or(url.len()));
    let path = if path.is_empty() { "/" } else { path };

    headers.add("Host".to_string(), addr.to_string());
    let headers_str: String = headers.into();
    let request_body = format!("GET {} HTTP/1.1\n{}\n\n", path, headers_str);

    let mut stream = TcpStream::connect(addr).map_err(|_| ConnectionFailed)?;
    stream
        .write(request_body.as_bytes())
        .map_err(|_| WriteError)?;

    let mut reader = BufReader::new(stream);
    let mut lines_iter = reader.by_ref().lines();

    let status_code: StatusCode = lines_iter
        .next()
        .ok_or(ResponseParseError("First line missing"))?
        .map_err(|_| ReadError)?
        .splitn(3, " ")
        .nth(1)
        .ok_or(ResponseParseError("Status code missing"))?
        .parse()
        .map_err(|_| ResponseParseError("Status code isn't a number"))?;

    let resp_headers: Headers = lines_iter
        .by_ref()
        .take_while(|x| x.as_ref().is_ok_and(|x| !x.is_empty()))
        .map(|x| {
            x.map_err(|_| ReadError).and_then(|x| {
                x.parse()
                    .map_err(|_| ResponseParseError("Couldn't parse header"))
            })
        })
        .collect::<Result<Vec<Header>, _>>()?
        .into();

    // the empty line between headers and body was consumed by .take_while

    let content_length: usize = resp_headers
        .get("Content-Length")
        .ok_or(ResponseParseError("No Content-Length header present"))
        .and_then(|x| {
            x.parse()
                .map_err(|_| ResponseParseError("Content-Length is not a number"))
        })?;

    let mut body = vec![0; content_length];
    let mut nread = 0;
    while nread < body.len() {
        match reader.read(&mut body[nread..]) {
            Ok(0) => break,
            Ok(n) => nread += n,
            Err(_) => return Err(ReadError),
        }
    }
    body.truncate(nread);

    Ok(HttpResponse {
        status_code,
        headers: resp_headers,
        body,
        premature_eof: nread < content_length,
    })
}
