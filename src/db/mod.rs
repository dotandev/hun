use rusqlite::{params, Connection};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use anyhow::Context;

#[derive(Debug)]
pub struct HistoryEntry {
    pub id: i64,
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub cwd: Option<String>,
    pub exit_code: Option<i32>,
    pub session_id: Option<String>,
    pub hostname: Option<String>,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn init() -> anyhow::Result<Self> {
        let db_path = get_db_path()?;
        let conn = Connection::open(&db_path).context("Failed to open database")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                cwd TEXT,
                exit_code INTEGER,
                session_id TEXT,
                hostname TEXT
            )",
            [],
        ).context("Failed to create table")?;

        Ok(Db { conn })
    }

    pub fn add_entry(&self, command: &str, cwd: Option<&str>, exit_code: Option<i32>, session_id: Option<&str>) -> anyhow::Result<()> {
        let now = Utc::now();
        let hostname = hostname::get().ok().and_then(|h| h.into_string().ok());

        self.conn.execute(
            "INSERT INTO history (command, timestamp, cwd, exit_code, session_id, hostname)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                command,
                now.timestamp(),
                cwd,
                exit_code,
                session_id,
                hostname
            ],
        ).context("Failed to insert entry")?;

        Ok(())
    }

    pub fn search_entries(&self, query: &str) -> anyhow::Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, timestamp, cwd, exit_code, session_id, hostname 
             FROM history 
             ORDER BY timestamp DESC 
             LIMIT 1000"
        )?;

        let rows = stmt.query_map([], |row| {
            let timestamp: i64 = row.get(2)?;
            Ok(HistoryEntry {
                id: row.get(0)?,
                command: row.get(1)?,
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_default(),
                cwd: row.get(3)?,
                exit_code: row.get(4)?,
                session_id: row.get(5)?,
                hostname: row.get(6)?,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        // Simple filtering for now, fuzzy match comes later in UI or here
        if !query.is_empty() {
            entries.retain(|e| e.command.contains(query));
        }

        Ok(entries)
    }

    pub fn get_stats(&self) -> anyhow::Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT command, COUNT(*) as count 
             FROM history 
             GROUP BY command 
             ORDER BY count DESC 
             LIMIT 10"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row?);
        }
        Ok(stats)
    }
}

fn get_db_path() -> anyhow::Result<PathBuf> {
    let mut path = dirs::data_dir().context("Could not find data directory")?;
    path.push("hun");
    std::fs::create_dir_all(&path)?;
    path.push("history.db");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_insertion_and_search() -> anyhow::Result<()> {
        // Use in-memory DB for testing
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                cwd TEXT,
                exit_code INTEGER,
                session_id TEXT,
                hostname TEXT
            )",
            [],
        )?;
        
        let db = Db { conn };

        db.add_entry("echo test", Some("/tmp"), Some(0), Some("session1"))?;
        db.add_entry("ls -la", Some("/home"), Some(1), Some("session1"))?;

        let results = db.search_entries("echo")?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "echo test");
        assert_eq!(results[0].exit_code, Some(0));

        let all_results = db.search_entries("")?;
        assert_eq!(all_results.len(), 2);

        Ok(())
    }
}
