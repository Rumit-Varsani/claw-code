//! Storage and persistence layer

use crate::{backend::Backend, config::ServerConfig, error::McpCliError, mcp::Message};
use rusqlite::{params, Connection, Result as SqliteResult};
use thiserror::Error;
use std::path::PathBuf;

/// Storage error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Storage for chat history and settings
pub struct Storage {
    path: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(path: PathBuf) -> Result<Self, StorageError> {
        let storage = Self { path };
        storage.initialize()?;

        Ok(storage)
    }

    /// Get default storage path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(std::path::PathBuf::new)
            .join("mcp-cli")
            .join("mcp-cli.db")
    }

    /// Initialize the database
    fn initialize(&self) -> Result<(), StorageError> {
        let conn = self.connection()?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                command TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS server_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                server_type TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                config_path TEXT,
                url TEXT,
                command TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS backend_config (
                backend TEXT NOT NULL,
                model TEXT NOT NULL,
                base_url TEXT,
                timeout INTEGER
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_role ON messages(role)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_role_timestamp ON messages(role, timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    /// Get a database connection
    fn connection(&self) -> Result<Connection, StorageError> {
        let conn = Connection::open(&self.path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(conn)
    }

    /// Save a message to history
    pub fn save_message(&self, message: &Message) -> Result<i64, StorageError> {
        let conn = self.connection()?;

        let result = conn.execute(
            "INSERT INTO messages (role, content, timestamp, command)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                message.role as &str,
                &message.content,
                message.timestamp,
                message.command.as_deref()
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get messages for a session
    pub fn get_messages(
        &self,
        role: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, StorageError> {
        let conn = self.connection()?;

        let mut query = self.base_message_query();

        if let Some(r) = role {
            query.push_str(" AND role = ?");
        }

        query.push_str(" ORDER BY timestamp DESC");
        query.push_str(&format!(" LIMIT ? OFFSET ?", limit, offset));

        let mut stmt = conn.prepare(&query)?;
        let mut rows = if let Some(r) = role {
            stmt.query_map(params![r], |row| self.row_to_message(row))?
        } else {
            stmt.query_map([], |row| self.row_to_message(row))?
        };

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
    }

    /// Get conversation history
    pub fn get_history(&self, limit: usize) -> Result<Vec<Message>, StorageError> {
        self.get_messages(None, limit, 0)
    }

    /// Clear chat history
    pub fn clear_history(&self) -> Result<(), StorageError> {
        let conn = self.connection()?;
        conn.execute("DELETE FROM messages", [])?;
        Ok(())
    }

    /// Save server configuration
    pub fn save_server(&self, server: &ServerConfig) -> Result<u64, StorageError> {
        let conn = self.connection()?;

        let result = conn.execute(
            "INSERT OR REPLACE INTO server_configs (name, server_type, enabled, config_path, url, command)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &server.name,
                &server.r#type,
                if server.enabled { 1 } else { 0 },
                server.config_path.as_deref(),
                server.url.as_deref(),
                server.stio_command.as_deref()
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get all server configurations
    pub fn get_servers(&self) -> Result<Vec<ServerConfig>, StorageError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare("SELECT name, server_type, enabled, config_path, url, command FROM server_configs")?;

        let mut rows = stmt.query_map([], |row| {
            Ok(ServerConfig {
                name: row.get(0)?,
                r#type: row.get(1)?,
                enabled: row.get(2)?,
                config_path: row.get(3)?,
                url: row.get(4)?,
                stdio_command: row.get(5)?,
            })
        })?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }

        Ok(servers)
    }

    /// Save backend configuration
    pub fn save_backend(&self, backend: &Backend) -> Result<u64, StorageError> {
        let conn = self.connection()?;

        let base_url = match backend {
            Backend::OpenAI { base_url, .. } => base_url.as_str(),
            Backend::Ollama {
                base_url, timeout, ..
            } => base_url.as_deref().unwrap_or("http://localhost:11434"),
        };

        let timeout = match backend {
            Backend::OpenAI { .. } => 30,
            Backend::Ollama { timeout, .. } => *timeout,
        };

        let result = conn.execute(
            "INSERT OR REPLACE INTO backend_config (backend, model, base_url, timeout)
             VALUES (?1, ?2, ?3, ?4)",
            params!["ollama", backend.model(), base_url, timeout],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get backend configuration
    pub fn get_backend(&self) -> Result<Option<Backend>, StorageError> {
        let conn = self.connection()?;
        let stmt = conn.prepare("SELECT backend, model, base_url, timeout FROM backend_config WHERE backend = 'ollama'")?;

        let mut row = stmt.query_row([], |row| {
            let backend: String = row.get(0)?;
            let model: String = row.get(1)?;
            let base_url: Option<String> = row.get(2)?;
            let timeout: u64 = row.get(3)?;

            match backend.as_str() {
                "ollama" => {
                    let be = Backend::Ollama {
                        base_url,
                        model,
                        timeout,
                    };
                    Ok(be)
                }
                _ => Ok(Backend::default()),
            }
        })?;

        Ok(Some(row))
    }

    /// Base query for messages
    fn base_message_query(&self) -> String {
        "SELECT role, content, timestamp, command FROM messages".to_string()
    }

    /// Convert row to message
    fn row_to_message(&self, row: &rusqlite::Row) -> rusqlite::Result<Message> {
        Ok(Message {
            role: row.get(0)?,
            content: row.get(1)?,
            timestamp: row.get(2)?,
            command: row.get(3)?,
        })
    }

    /// Close the storage (called when done)
    pub fn close(&self) -> Result<(), StorageError> {
        if let Ok(conn) = self.connection() {
            conn.pragma("optimize", [])?;
        }
        Ok(())
    }
}

/// Storage Manager for managing multiple storage instances
pub struct StorageManager {
    main_storage: Storage,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new() -> Result<Self, StorageError> {
        let storage = Storage::new(Storage::default_path())?;
        Ok(Self { main_storage: storage })
    }

    /// Get the main storage
    pub fn storage(&self) -> &Storage {
        &self.main_storage
    }

    /// Get mutable reference to main storage
    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.main_storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_message() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().join("test.db")).unwrap();

        let msg = Message {
            role: crate::mcp::types::MessageRole::User,
            content: "Hello, MCP".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            command: None,
        };

        storage.save_message(&msg).unwrap();

        let messages = storage.get_messages(None, 10, 0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello, MCP");
    }

    #[test]
    fn test_message_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().join("test.db")).unwrap();

        let user_msg = Message {
            role: crate::mcp::types::MessageRole::User,
            content: "User message".to_string(),
            timestamp: 1,
            command: None,
        };

        let ai_msg = Message {
            role: crate::mcp::types::MessageRole::Ai,
            content: "AI message".to_string(),
            timestamp: 2,
            command: None,
        };

        storage.save_message(&user_msg).unwrap();
        storage.save_message(&ai_msg).unwrap();

        let messages = storage.get_messages(Some("user"), 10, 0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "User message");
    }
}