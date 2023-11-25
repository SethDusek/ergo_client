pub mod endpoints;
pub mod extensions;

use self::{endpoints::NodeEndpoint, extensions::NodeExtension};
use crate::Error;
use reqwest::{
    header::{HeaderMap, InvalidHeaderValue},
    Client, Url,
};
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum NodeError {
    #[error("Nodes wallet doesn't hold enough nanoergs, {found} < {requested}")]
    InsufficientFunds { requested: u64, found: u64 },

    #[error("Specified API key is not a valid header value")]
    InvalidApiKey(#[from] InvalidHeaderValue),
}

#[derive(Debug, Clone)]
pub struct NodeClient {
    endpoints: NodeEndpoint,
}

impl NodeClient {
    pub fn from_url_str(url_str: &str, api_key: String, timeout: Duration) -> Result<Self, Error> {
        let url = Url::parse(url_str).map_err(|e| Error::UrlParsing(e.to_string()))?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "api_key",
            api_key.clone().try_into().map_err(NodeError::from)?,
        );
        let client = Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            // outputs all connection events if `trace` log level is set for `reqwest` crate
            // useful to debug response errors
            .connection_verbose(true)
            .build()
            .map_err(|e| Error::BuildClient(e))?;
        Ok(Self {
            endpoints: NodeEndpoint::new(client, url)?,
        })
    }

    pub fn endpoints(&self) -> &NodeEndpoint {
        &self.endpoints
    }

    pub fn extensions(&self) -> NodeExtension {
        NodeExtension::new(&self.endpoints)
    }
}
