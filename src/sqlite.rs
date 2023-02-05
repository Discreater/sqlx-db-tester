use std::{env::temp_dir, fs, path::PathBuf};

use sqlx::{
    migrate::{MigrationSource, Migrator},
    sqlite::SqliteConnectOptions,
    Connection, SqliteConnection, SqlitePool,
};
use std::{path::Path, thread};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct TestSqlite;

impl TestSqlite {
    pub fn new<S>(migrations: S) -> Self
    where
        S: MigrationSource<'static> + Send + Sync + 'static,
    {
        let tdb = Self {};
        // remove temp db if exists
        let db_file = Self::db_temp_file();
        if db_file.exists() {
            if db_file.is_dir() {
                fs::remove_dir_all(&db_file).unwrap();
            } else {
                fs::remove_file(&db_file).unwrap();
            }
        }
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                // connect to test database for migration
                let mut conn = SqliteConnection::connect_with(&Self::option())
                    .await
                    .unwrap();
                let m = Migrator::new(migrations).await.unwrap();
                m.run(&mut conn).await.unwrap();
            });
        })
        .join()
        .expect("failed to create database");

        tdb
    }

    fn db_temp_file() -> PathBuf {
        temp_dir().join("sqlx-db-tester.db")
    }

    fn option() -> SqliteConnectOptions {
        SqliteConnectOptions::new()
            .filename(Self::db_temp_file())
            .create_if_missing(true)
    }

    pub async fn get_pool(&self) -> SqlitePool {
        SqlitePool::connect_with(Self::option()).await.unwrap()
    }
}

impl Default for TestSqlite {
    fn default() -> Self {
        Self::new(Path::new("./migrations"))
    }
}

#[cfg(test)]
mod tests {
    use crate::TestSqlite;

    #[tokio::test]
    async fn test_sqlite_should_create_and_drop() {
        let tdb = TestSqlite::default();
        let pool = tdb.get_pool().await;
        println!("!!!");
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
