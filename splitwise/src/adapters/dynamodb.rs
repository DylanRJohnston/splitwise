use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest},
    Client,
};
use color_eyre::eyre::{ContextCompat, Result, WrapErr};
use tracing::instrument;

use serde::{Deserialize, Serialize};

use crate::ports::store::{Storable, Store};

#[derive(Debug)]
pub struct DynamoDB {
    client: Client,
    table_name: String,
}

impl DynamoDB {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(sdk_config: &SdkConfig, table_name: &str) -> DynamoDB {
        DynamoDB {
            client: Client::new(sdk_config),
            table_name: table_name.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Id {
    id: String,
}

#[async_trait]
impl Store for DynamoDB {
    #[instrument]
    async fn has(&self, id: String) -> Result<bool> {
        Ok(self.batch_has(&[id.clone()]).await?.contains(&id))
    }

    #[instrument]
    async fn add<A: Storable>(&self, item: A) -> Result<()> {
        self.batch_add(&[item]).await
    }

    #[instrument]
    async fn batch_has(&self, ids: &[String]) -> Result<HashSet<String>> {
        if ids.is_empty() {
            return Ok(HashSet::new());
        }

        let keys_and_attributes = ids
            .iter()
            .map(|it| HashMap::from([("id".to_owned(), AttributeValue::S(it.to_owned()))]))
            .fold(KeysAndAttributes::builder(), |it, key| it.keys(key))
            .build();

        let mut response = self
            .client
            .batch_get_item()
            .request_items(self.table_name.clone(), keys_and_attributes)
            .send()
            .await
            .wrap_err("Failed to talk to DynamoDB")?
            .responses
            .wrap_err("No responses")?;

        let data = response
            .remove(&self.table_name)
            .wrap_err(format!("No data for table {}", self.table_name))?;

        let ids = serde_dynamo::from_items::<_, Id>(data.clone())
            .wrap_err_with(|| format!("Failed to deserialize {:?}", data))?;

        Ok(ids.into_iter().map(|it| it.id).collect())
    }

    #[instrument]
    async fn batch_add<A: Storable>(&self, items: &[A]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let to_write_request = |item: &A| -> Result<WriteRequest> {
            let data = serde_dynamo::to_item(item)
                .wrap_err_with(|| format!("Failed to serialize {:?}", item))?;

            let put_request = PutRequest::builder().set_item(Some(data)).build();
            let write_request = WriteRequest::builder()
                .set_put_request(Some(put_request))
                .build();

            Ok(write_request)
        };

        let write_requests = items
            .iter()
            .map(to_write_request)
            .collect::<Result<Vec<_>>>()?;

        self.client
            .batch_write_item()
            .request_items(self.table_name.to_owned(), write_requests)
            .send()
            .await
            .wrap_err("Failed to send items to DynamoDB")?;

        Ok(())
    }
}
