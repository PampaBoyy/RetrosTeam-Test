//! SQLite access and versioned migrations (tracked through PRAGMA user_version).
use rusqlite::Connection;

const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../migrations/0001_init.sql"))];

pub fn latest_version() -> i64 {
    MIGRATIONS.last().map(|m| m.0).unwrap_or(0)
}

/// Apply pending migrations, each in its own transaction. Returns the resulting version.
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<i64> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let current: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (version, sql) in MIGRATIONS {
        if *version > current {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
    }
    conn.pragma_query_value(None, "user_version", |r| r.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_fresh_db_to_latest() {
        let mut c = Connection::open_in_memory().unwrap();
        assert_eq!(migrate(&mut c).unwrap(), latest_version());
        c.execute("INSERT INTO games (title, file_path) VALUES ('t','/a')", []).unwrap();
    }
    #[test]
    fn migrate_is_idempotent_and_keeps_data() {
        let mut c = Connection::open_in_memory().unwrap();
        migrate(&mut c).unwrap();
        c.execute("INSERT INTO games (title, file_path) VALUES ('t','/a')", []).unwrap();
        migrate(&mut c).unwrap();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
    }
    #[test]
    fn duplicate_file_path_is_rejected() {
        let mut c = Connection::open_in_memory().unwrap();
        migrate(&mut c).unwrap();
        c.execute("INSERT INTO games (title, file_path) VALUES ('a','/x')", []).unwrap();
        assert!(c.execute("INSERT INTO games (title, file_path) VALUES ('b','/x')", []).is_err());
    }
}
