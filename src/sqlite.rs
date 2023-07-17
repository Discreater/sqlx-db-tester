use std::{env::temp_dir, fs, path::PathBuf};

use sqlx::{
    migrate::{MigrationSource, Migrator},
    sqlite::SqliteConnectOptions,
    Connection, SqliteConnection, SqlitePool,
};
use std::{path::Path, thread};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct TestSqlite(SqlitePool);

impl TestSqlite {
    pub async fn new<S>(migrations: S) -> Self
    where
        S: MigrationSource<'static> + Send + Sync + 'static,
    {
        // remove temp db if exists
        let db_file = Self::db_temp_file();
        if db_file.exists() {
            if db_file.is_dir() {
                fs::remove_dir_all(&db_file).unwrap();
            } else {
                fs::remove_file(&db_file).unwrap();
            }
        }
        let mut pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        let m = Migrator::new(migrations).await.unwrap();
        m.run(&pool).await.unwrap();
        Self(pool)
    }

    fn db_temp_file() -> PathBuf {
        temp_dir().join("sqlx-db-tester.db")
    }

    fn option() -> SqliteConnectOptions {
        SqliteConnectOptions::new()
            .filename(Self::db_temp_file())
            .create_if_missing(true)
    }

    pub fn get_pool(self) -> SqlitePool {
        self.0
    }

    pub async fn default() -> Self {
        Self::new(Path::new("./fixtures/migrations")).await
    }
}

#[cfg(test)]
mod tests {
    use crate::TestSqlite;

    #[tokio::test]
    async fn test_sqlite_should_create_and_drop() {
        let tdb = TestSqlite::default().await;
        let pool = tdb.get_pool();
        // insert todo
        sqlx::query("INSERT INTO todos (title) VALUES ('test')")
            .execute(&pool)
            .await
            .unwrap();
        // get todo
        let (id, title) = sqlx::query_as::<_, (i32, String)>("SELECT id, title FROM todos")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(id, 0);
        assert_eq!(title, "test");
    }
}
