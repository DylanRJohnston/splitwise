use anyhow::Result;
use fake::{Dummy, Fake, Faker};
use serde::{Deserialize, Serialize};
use splitwise::ports::store::Store;
use splitwise::{adapters::dynamodb::DynamoDB, models::ID};

#[derive(Debug, Dummy, Deserialize, Serialize)]
struct Test {
    id: String,
}

impl ID for Test {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

#[tokio::test]
async fn dynamodb_test() -> Result<()> {
    let client = DynamoDB::new("splitwise_integration_test".to_owned()).await;
    let data: Vec<Test> = vec![Faker.fake(), Faker.fake(), Faker.fake()];
    let set = client.as_set();

    set.batch_add(&data).await?;

    let result = set.batch_has(&data).await?;

    assert!(result.contains_key(&data[0].id()));
    assert!(result.contains_key(&data[1].id()));
    assert!(result.contains_key(&data[2].id()));

    Ok(())
}
