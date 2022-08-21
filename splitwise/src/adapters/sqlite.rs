use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::ports::store::Set;

pub struct SQLite {
    connection: Mutex<Connection>,
}

impl SQLite {
    pub fn new<A>(filename: &str) -> Result<impl Set<A>> {
        let connection = Connection::open(filename)?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS expenses (ID INTEGER UNIQUE NOT NULL PRIMARY KEY);",
            [],
        )?;

        Ok(SQLite {
            connection: Mutex::new(connection),
        })
    }
}

#[async_trait]
impl<A> Set<A> for SQLite {
    async fn add(&self, key: A) -> Result<()> {
        let connection = self.connection.lock().await;

        let mut cursor = connection
            .execute("INSERT INTO expenses (ID) VALUES (?)")?
            .into_cursor();

        cursor.bind(&[Value::Integer(key)])?;
        cursor.next()?;

        Ok(())
    }

    async fn has(&self, key: A) -> Result<bool> {
        let connection = self.connection.lock().await;

        let mut cursor = connection
            .prepare("SELECT id FROM expenses WHERE id = ?")?
            .into_cursor();

        cursor.bind(&[Value::Integer(key)])?;

        while let Some(_) = cursor.next()? {
            return Ok(true);
        }

        Ok(false)
    }

    // has is fast enough with a local sqlite, a batch hash isn't worth the effort
    async fn batch_has(&self, keys: &[A]) -> Result<HashSet<A>> {
        let mut result = HashSet::new();

        for key in keys {
            if self.has(*key).await? {
                result.insert(*key);
            }
        }

        Ok(result)
    }

    async fn batch_add(&self, keys: &[A]) -> Result<()> {
        if keys.len() == 0 {
            return Ok(());
        };

        let mut statement = String::new();

        statement += "INSERT INTO expenses (ID) values ";
        for _ in 1..keys.len() {
            statement += "(?), ";
        }
        statement += "(?);";

        println!("{}", statement);

        let connection = self.connection.lock().await;
        let mut cursor = connection.prepare(statement)?.into_cursor();

        let bindings = keys
            .into_iter()
            .map(|key| Value::Integer(*key))
            .collect::<Vec<Value>>();

        println!("{:?}", bindings);

        cursor.bind(&bindings)?;
        cursor.next()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::ports::store::Set;

    use super::SQLite;

    #[tokio::test]
    async fn test_has_true() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.add(420).await?;

        assert!(set.has(420).await?);
        assert_eq!(set.has(69).await?, false);

        Ok(())
    }

    #[tokio::test]
    async fn test_has_false() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.add(420).await?;
        assert_eq!(set.has(69).await?, false);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_add() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.batch_add(&vec![420, 69, 3]).await?;

        assert!(set.has(420).await?);
        assert!(set.has(69).await?);
        assert!(set.has(3).await?);
        assert!(set.has(21).await? == false);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_has() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.batch_add(&vec![420, 69, 3]).await?;
        let result = set.batch_has(&vec![420, 69, 3]).await?;

        assert!(result.contains(&420));
        assert!(result.contains(&69));
        assert!(result.contains(&3));
        assert!(result.contains(&21) == false);

        Ok(())
    }
}
