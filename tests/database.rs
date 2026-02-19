use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub struct Database {
    url: String,
}

impl Database {
    pub fn new(url: &str) -> Self {
        Database { url: url.into() }
    }

    pub fn create(self) -> Self {
        let (database, postgres_url) = self.split_url();
        let mut conn = PgConnection::establish(&postgres_url).unwrap();
        diesel::sql_query(format!(r#"DROP DATABASE IF EXISTS "{}""#, database))
            .execute(&mut conn)
            .unwrap();
        diesel::sql_query(format!(r#"CREATE DATABASE "{}""#, database))
            .execute(&mut conn)
            .unwrap();
        self
    }

    pub fn conn(&self) -> PgConnection {
        PgConnection::establish(&self.url)
            .expect(&format!("failed to establish connection to {}", &self.url))
    }

    fn split_url(&self) -> (String, String) {
        let mut split: Vec<&str> = self.url.split('/').collect();
        let database = split.pop().unwrap();

        // assumes admin database is named 'postgres'
        let postgres_url = format!("{}/{}", split.join("/"), "postgres");
        (database.into(), postgres_url)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let (database, postgres_url) = self.split_url();
        let mut conn = PgConnection::establish(&postgres_url).unwrap();
        diesel::sql_query(format!(r#"DROP DATABASE IF EXISTS "{}""#, database))
            .execute(&mut conn)
            .unwrap();
        println!("database dropped");
    }
}

// migrations

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(conn: &mut PgConnection) {
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(_) => {
            println!("Database migrated");
        }
        Err(e) => {
            eprint!("Error migrating database: {}", e);
        }
    };
}
