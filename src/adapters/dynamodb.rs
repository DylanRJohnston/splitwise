use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_sdk_dynamodb::{
    model::{KeysAndAttributes, PutRequest, WriteRequest},
    Client,
};
use aws_types::sdk_config::SdkConfig;
use serde::{de::DeserializeOwned, Serialize};

use crate::{models::ID, ports::store::Set};

pub struct DynamoDB {
    client: Client,
    table_name: String,
}

impl DynamoDB {
    pub fn new(table_name: String) -> DynamoDB {
        let config = SdkConfig::builder().build();
        let client = Client::new(&config);

        DynamoDB { client, table_name }
    }
}

#[async_trait]
impl<A: Serialize + DeserializeOwned + ID + Send + Sync + 'static> Set<A> for DynamoDB {
    async fn has(&self, key: A) -> Result<bool> {
        let id = key.id();

        Ok(self.batch_has(&[key]).await?.contains_key(&id))
    }

    async fn add(&self, key: A) -> Result<()> {
        self.batch_add(&[key]).await
    }

    async fn batch_has(&self, key: &[A]) -> Result<HashMap<A::ID, A>> {
        let keys_and_attributes = KeysAndAttributes::builder().build();

        let mut response = self
            .client
            .batch_get_item()
            .request_items(self.table_name.to_owned(), keys_and_attributes)
            .send()
            .await?
            .responses
            .ok_or(anyhow!("no response"))?;

        let data = response
            .remove(&self.table_name)
            .ok_or(anyhow!("no data for table {}", self.table_name))
            .and_then(|it| Ok(serde_dynamo::from_items::<_, A>(it)?))?;

        Ok(data.into_iter().map(|it| (it.id(), it)).collect())
    }

    async fn batch_add(&self, key: &[A]) -> Result<()> {
        let to_write_request = |item: &A| -> Result<WriteRequest> {
            let data = serde_dynamo::to_item(item)?;

            let put_request = PutRequest::builder().set_item(Some(data)).build();
            let write_request = WriteRequest::builder()
                .set_put_request(Some(put_request))
                .build();

            Ok(write_request)
        };

        let items = key
            .iter()
            .map(to_write_request)
            .collect::<Result<Vec<_>>>()?;

        self.client
            .batch_write_item()
            .request_items(self.table_name.to_owned(), items)
            .send()
            .await?;

        Ok(())
    }
}
