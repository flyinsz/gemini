use std::str::FromStr;

use console_error_panic_hook::set_once as set_panic_hook;
use futures_util::StreamExt;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use tokio::sync::OnceCell;
use worker::*;

static CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn get_client() -> Client {
    CLIENT
        .get_or_init(|| async { Client::builder().build().expect("Failed to create client") })
        .await
        .clone()
}

#[event(fetch)]
async fn main(mut req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    // Replace host with the backend host
    let mut url = req.url()?;
    url.set_host(Some("generativelanguage.googleapis.com"))?;
    url.set_scheme("https")
        .map_err(|_| worker::Error::RustError(format!("Failed to set scheme")))?;

    // Convert method
    let method = match req.method() {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
        Method::Head => reqwest::Method::HEAD,
        Method::Connect => reqwest::Method::CONNECT,
        Method::Options => reqwest::Method::OPTIONS,
        Method::Trace => reqwest::Method::TRACE,
        Method::Patch => reqwest::Method::PATCH,
    };

    // Convert headers
    let mut headers = HeaderMap::new();
    for (k, v) in req.headers().into_iter() {
        headers.insert(
            HeaderName::from_str(k.as_str())
                .map_err(|_| worker::Error::RustError(format!("Failed to parse header name")))?,
            HeaderValue::from_str(v.as_str())
                .map_err(|_| worker::Error::RustError(format!("Failed to parse header value")))?,
        );
    }

    // Send request
    let resp = get_client()
        .await
        .request(method, url)
        .headers(headers)
        .body(req.bytes().await?)
        .send()
        .await
        .map_err(|_| worker::Error::RustError(format!("Failed to send request")))?;

    // Convert response
    let status = resp.status().as_u16();
    let mut worker_headers = worker::Headers::new();
    resp.headers().iter().for_each(|(k, v)| {
        if let Ok(value) = v.to_str() {
            let _ = worker_headers.append(k.as_str(), value);
        }
    });

    // Convert stream body
    let stream = async_stream::stream! {
        let mut bytes_streams = resp.bytes_stream();
        while let Some(s) = bytes_streams.next().await {
            yield s.map_err(|_| worker::Error::RustError(format!("Failed to read response")));
        }
    };

    // Return response
    let response = Response::from_stream(stream)?
        .with_status(status)
        .with_headers(worker_headers);
    Ok(response)
}
