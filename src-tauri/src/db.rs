use crate::core::Interval;
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct Database {
    pub conn: Connection,
}
impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS intervals (id TEXT PRIMARY KEY, source TEXT NOT NULL, key TEXT NOT NULL, label TEXT NOT NULL, domain TEXT, started_at INTEGER NOT NULL, ended_at INTEGER NOT NULL, idle INTEGER NOT NULL DEFAULT 0); CREATE INDEX IF NOT EXISTS idx_intervals_time ON intervals(started_at, ended_at); CREATE TABLE IF NOT EXISTS rules (source TEXT NOT NULL, key TEXT NOT NULL, category TEXT NOT NULL, daily_limit INTEGER, PRIMARY KEY(source,key));")?;
        Ok(())
    }
    pub fn insert(
        &self,
        id: &str,
        item: &Interval,
        label: &str,
        domain: Option<&str>,
        idle: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO intervals VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                id,
                item.source,
                item.key,
                label,
                domain,
                item.start,
                item.end,
                idle as i32
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_creates_tracking_tables() {
        let db = Database::in_memory().expect("in-memory database");
        let count: i64 = db.conn.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('intervals','rules')", [], |row| row.get(0)).expect("schema query");
        assert_eq!(count, 2);
    }
}
