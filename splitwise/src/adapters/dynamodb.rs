use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest},
    Client,
};

use serde::{Deserialize, Serialize};

use crate::ports::store::{Storable, Store};

pub struct DynamoDB {
    client: Client,
    table_name: String,
}

impl DynamoDB {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(table_name: String) -> DynamoDB {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-2");
        let config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&config);

        DynamoDB { client, table_name }
    }
}

#[derive(Serialize, Deserialize)]
struct Id {
    id: String,
}

#[async_trait]
impl Store for DynamoDB {
    async fn has(&self, id: String) -> Result<bool> {
        Ok(self.batch_has(&[id.clone()]).await?.contains(&id))
    }

    async fn add<A: Storable>(&self, item: A) -> Result<()> {
        self.batch_add(&[item]).await
    }

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
            .await?
            .responses
            .ok_or_else(|| anyhow!("no response"))?;

        let data = response
            .remove(&self.table_name)
            .ok_or_else(|| anyhow!("no data for table {}", self.table_name))
            .and_then(|it| Ok(serde_dynamo::from_items::<_, Id>(it)?))?;

        Ok(data.into_iter().map(|it| it.id).collect())
    }

    async fn batch_add<A: Storable>(&self, items: &[A]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let to_write_request = |item: &A| -> Result<WriteRequest> {
            let data = serde_dynamo::to_item(item)?;

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
            .await?;

        Ok(())
    }
}
