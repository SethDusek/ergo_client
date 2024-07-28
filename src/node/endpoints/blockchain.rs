use ergo_lib::{
    chain::transaction::{Transaction, TxId},
    ergotree_ir::chain::ergo_box::{BoxId, ErgoBox},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::node::{process_response, NodeError};

#[derive(Serialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexQuery {
    pub offset: u64,
    pub limit: u32,
    pub sort_direction: SortDirection,
    pub include_unconfirmed: bool,
}

#[derive(Deserialize, Debug)]
pub struct IndexedBox {
    #[serde(flatten)]
    pub ergo_box: ErgoBox,
    #[serde(rename = "globalIndex")]
    pub global_index: u64,
    #[serde(rename = "spentTransactionId")]
    pub spent_transaction_id: Option<TxId>,
}

#[derive(Debug, Clone)]
pub struct BlockchainEndpoint<'a> {
    client: &'a Client,
    url: Url,
}

impl<'a> BlockchainEndpoint<'a> {
    pub fn new(client: &'a Client, mut url: Url) -> Result<Self, NodeError> {
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("blockchain");
        Ok(Self { client, url })
    }
}

impl<'a> BlockchainEndpoint<'a> {
    pub async fn unspent_by_address(
        &self,
        address: &str,
        query: IndexQuery,
    ) -> Result<Vec<IndexedBox>, NodeError> {
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .extend(&["box", "unspent", "byAddress"]);
        process_response(
            self.client
                .post(url)
                .query(&query)
                .json(address)
                .send()
                .await
                .map_err(NodeError::Http)?,
        )
        .await
    }

    pub async fn get_transaction_by_id(&self, tx_id: &TxId) -> Result<Transaction, NodeError> {
        let mut url = self.url.clone();
        let tx_id = tx_id.to_string();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .extend(&["transaction", "byId", &tx_id]);
        process_response(self.client.get(url).send().await.map_err(NodeError::Http)?).await
    }

    pub async fn get_box_by_id(&self, box_id: &BoxId) -> Result<IndexedBox, NodeError> {
        let mut url = self.url.clone();
        let box_id = box_id.to_string();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .extend(&["box", "byId", &box_id]);
        process_response(self.client.get(url).send().await.map_err(NodeError::Http)?).await
    }
}
