use anyhow::Result;
use sqlite::{Connection, Value};

use crate::ports::store::Set;

pub struct SQLite {
    connection: Connection,
}

impl SQLite {
    pub fn new(filename: &str) -> Result<impl Set> {
        let connection = Connection::open(filename)?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS expenses (ID INTEGER UNIQUE NOT NULL PRIMARY KEY);",
        )?;

        Ok(SQLite { connection })
    }
}

impl Set for SQLite {
    fn add(&self, key: i64) -> Result<()> {
        let mut cursor = self
            .connection
            .prepare("INSERT INTO expenses (ID) VALUES (?)")?
            .into_cursor();

        cursor.bind(&[Value::Integer(key)])?;
        cursor.next()?;

        Ok(())
    }

    fn has(&self, key: i64) -> Result<bool> {
        let mut cursor = self
            .connection
            .prepare("SELECT id FROM expenses WHERE id = ?")?
            .into_cursor();

        cursor.bind(&[Value::Integer(key)])?;

        while let Some(_) = cursor.next()? {
            return Ok(true);
        }

        Ok(false)
    }

    fn batch_add(&self, keys: Vec<i64>) -> Result<()> {
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

        let mut cursor = self.connection.prepare(statement)?.into_cursor();

        let bindings = &keys
            .into_iter()
            .map(|key| Value::Integer(key))
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

    #[test]
    fn test_has_true() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.add(420)?;

        assert!(set.has(420)?);
        assert_eq!(set.has(69)?, false);

        Ok(())
    }

    #[test]
    fn test_has_false() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.add(420)?;
        assert_eq!(set.has(69)?, false);

        Ok(())
    }

    #[test]
    fn test_batch_add() -> Result<()> {
        let set = SQLite::new(":memory:")?;

        set.batch_add(vec![420, 69, 3])?;

        assert!(set.has(420)?);
        assert!(set.has(69)?);
        assert!(set.has(3)?);

        Ok(())
    }
}
