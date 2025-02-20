use std::env;

use bon::bon;
use http::{HeaderMap, HeaderValue};
use reqwest::Client;
use thiserror::Error;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// A client for interacting with the DeepInfra API.
///
/// This struct encapsulates an HTTP client with default settings and headers required for authentication.
#[derive(Clone, Debug)]
pub struct DeepinfraClient {
    /// The underlying HTTP client used for sending requests.
    pub(crate) client: Client,
}

/// Errors that can occur when building a DeepinfraClient.
#[derive(Error, Debug)]
pub enum DeepinfraClientBuilderError {
    /// Indicates that building the HTTP client failed.
    #[error("Could not build client {0}")]
    ReqwestError(#[from] reqwest::Error),
    /// Indicates that an invalid header value was provided.
    #[error("Invalid header value {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
}

#[bon]
impl DeepinfraClient {
    /// Creates a new instance of DeepinfraClient.
    ///
    /// # Example
    ///
    /// ```
    /// use deepinfra_client_rs::client::DeepinfraClient;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let token = "your_api_token";
    ///     let client = DeepinfraClient::new(token)?;
    ///     // Use client for further API calls...
    ///     Ok(())
    /// }
    /// ```
    #[builder]
    pub fn new(token: &str) -> Result<Self, DeepinfraClientBuilderError> {
        // Create headers with authorization token.
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {token}");
        headers.insert("Authorization", HeaderValue::from_str(&bearer)?);

        // Create a client with default headers and user agent.
        let client = Client::builder()
            .default_headers(headers)
            .user_agent(APP_USER_AGENT)
            .build()?;

        // Return the constructed DeepinfraClient.
        Ok(DeepinfraClient { client })
    }
}
