use ergo_lib::{chain::transaction::Transaction, ergo_chain_types::BlockId};
use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::node::{process_response, NodeError};

#[derive(Debug, Clone)]
pub struct BlocksEndpoint<'a> {
    client: &'a Client,
    url: Url,
}

impl<'a> BlocksEndpoint<'a> {
    pub fn new(client: &'a Client, mut url: Url) -> Result<Self, NodeError> {
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .push("blocks");
        Ok(Self { client, url })
    }

    pub async fn transactions(&self, block_id: &BlockId) -> Result<Vec<Transaction>, NodeError> {
        #[derive(Deserialize)]
        struct BlockTransactions {
            transactions: Vec<Transaction>,
        }
        let mut url = self.url.clone();
        url.path_segments_mut()
            .map_err(|_| NodeError::BaseUrl)?
            .extend(&[&block_id.to_string(), "transactions"]);
        Ok(process_response::<BlockTransactions>(
            self.client.get(url).send().await.map_err(NodeError::Http)?,
        )
        .await?
        .transactions)
    }
}
