pub mod boxes;
pub mod transaction;

use self::{boxes::BoxesEndpoint, transaction::TransactionEndpoint};
use crate::node::{process_response, NodeError};
use ergo_lib::{
    ergotree_interpreter::sigma_protocol::private_input::DlogProverInput,
    ergotree_ir::chain::address::NetworkAddress,
};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct WalletEndpoint<'a> {
    client: &'a Client,
    url: Url,
}

impl<'a> WalletEndpoint<'a> {
    pub fn new(client: &'a Client, mut url: Url) -> Result<Self, NodeError> {
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("wallet");
        Ok(Self { client, url })
    }

    pub fn transaction(&self) -> Result<TransactionEndpoint, NodeError> {
        TransactionEndpoint::new(&self.client, self.url.clone())
    }

    pub fn boxes(&self) -> Result<BoxesEndpoint, NodeError> {
        BoxesEndpoint::new(&self.client, self.url.clone())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_initialized: bool,
    pub is_unlocked: bool,
    pub change_address: String,
    pub wallet_height: i64,
    pub error: String,
}

impl<'a> WalletEndpoint<'a> {
    pub async fn status(&self) -> Result<StatusResponse, NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("status");
        process_response(self.client.get(url).send().await.map_err(NodeError::Http)?).await
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockRequest {
    pass: String,
}

impl<'a> WalletEndpoint<'a> {
    pub async fn get_addresses(&self) -> Result<Vec<NetworkAddress>, NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("addresses");
        process_response(self.client.get(url).send().await.map_err(NodeError::Http)?).await
    }

    pub async fn rescan(&self, from_height: u32) -> Result<(), NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("rescan");
        process_response::<String>(
            self.client
                .post(url)
                .json(&json!({ "fromHeight": from_height }))
                .send()
                .await
                .map_err(NodeError::Http)?,
        )
        .await
        .map(|_| ())
    }

    pub async fn unlock(&self, password: String) -> Result<(), NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("unlock");
        let body = UnlockRequest { pass: password };
        // Respods with a string "OK"
        process_response::<String>(
            self.client
                .post(url)
                .json(&body)
                .send()
                .await
                .map_err(NodeError::Http)?,
        )
        .await?;
        Ok(())
    }

    pub async fn get_private_key(
        &self,
        address: &NetworkAddress,
    ) -> Result<DlogProverInput, NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("getPrivateKey");
        let body = serde_json::json!({"address": address.to_base58()});
        process_response(
            self.client
                .post(url)
                .json(&body)
                .send()
                .await
                .map_err(NodeError::Http)?,
        )
        .await
    }
}
