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
    let ids = data.iter().map(ID::id).collect::<Vec<_>>();

    client.batch_add(&data).await?;

    let result = client.batch_has(&ids).await?;

    assert!(result.contains(&ids[0]));
    assert!(result.contains(&ids[1]));
    assert!(result.contains(&ids[2]));

    Ok(())
}
