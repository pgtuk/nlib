use rusqlite::{params, Connection, Result};
use std::collections::HashSet;

struct Migration {
    version: u32,
    description: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "create books table",
    sql: "CREATE TABLE IF NOT EXISTS  books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                description TEXT,
                cover_url TEXT,
                read_by_nika BOOLEAN NOT NULL DEFAULT 0,
                read_by_petya BOOLEAN NOT NULL DEFAULT 0
            );",
}];

pub(super) fn migrate(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        [],
    )?;

    let applied: HashSet<u32> = {
        let mut stmt = conn.prepare("SELECT version FROM _schema_migrations")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.filter_map(std::result::Result::ok).collect()
    };

    for migration in MIGRATIONS {
        if !applied.contains(&migration.version) {
            let tx = conn.transaction()?;
            tx.execute_batch(migration.sql)?;
            tx.execute(
                "INSERT INTO _schema_migrations (version, description) VALUES (?1, ?2);",
                params![migration.version, migration.description],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();

        // Verify _schema_migrations table has recorded version 1
        let applied_version: u32 = conn
            .query_row(
                "SELECT version FROM _schema_migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(applied_version, 1);

        // Verify books table structure
        {
            let mut stmt = conn.prepare("PRAGMA table_info(books)").unwrap();
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .filter_map(std::result::Result::ok)
                .collect();

            assert!(columns.contains(&"id".to_string()));
            assert!(columns.contains(&"title".to_string()));
            assert!(columns.contains(&"author".to_string()));
            assert!(columns.contains(&"description".to_string()));
            assert!(columns.contains(&"cover_url".to_string()));
            assert!(columns.contains(&"read_by_nika".to_string()));
            assert!(columns.contains(&"read_by_petya".to_string()));
        }

        // Running migrate again should be an idempotent no-op and succeed
        migrate(&mut conn).unwrap();
    }
}
