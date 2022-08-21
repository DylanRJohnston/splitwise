use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{
    model::{AttributeValue, KeysAndAttributes, PutRequest, WriteRequest},
    Client,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{models::ID, ports::store::Set};

pub struct DynamoDB {
    client: Client,
    table_name: String,
}

pub trait Storable = Serialize + DeserializeOwned + ID + Send + Sync + 'static;

impl DynamoDB {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new<A: Storable>(table_name: String) -> impl Set<A> {
        let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-2");
        let config = aws_config::from_env().region(region_provider).load().await;

        let client = Client::new(&config);

        DynamoDB { client, table_name }
    }
}

#[async_trait]
impl<A: Storable> Set<A> for DynamoDB {
    async fn has(&self, key: A) -> Result<bool> {
        let id = key.id();

        Ok(self.batch_has(&[key]).await?.contains_key(&id))
    }

    async fn add(&self, key: A) -> Result<()> {
        self.batch_add(&[key]).await
    }

    async fn batch_has(&self, keys: &[A]) -> Result<HashMap<String, A>> {
        let keys_and_attributes = keys
            .iter()
            .map(|it| HashMap::from([("id".to_owned(), AttributeValue::S(it.id()))]))
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

        println!("Write Request\n\n{:?}\n\n", items);

        self.client
            .batch_write_item()
            .request_items(self.table_name.to_owned(), items)
            .send()
            .await?;

        Ok(())
    }
}
