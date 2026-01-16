//! Discovery logging for measuring search effectiveness
//!
//! Logs query_tools calls to help identify:
//! - Zero-result queries (missing coverage)
//! - Repeated searches (user hunting for terms)
//! - Query patterns for improving discovery

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// A single discovery query log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryLogEntry {
    /// Timestamp of the query
    pub timestamp: DateTime<Utc>,
    /// Session ID for correlating queries (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// The search query
    pub query: String,
    /// Number of results requested
    pub top_k: usize,
    /// Tool names returned
    pub results: Vec<String>,
    /// Number of results returned
    pub result_count: usize,
    /// Query duration in milliseconds
    pub query_duration_ms: u64,
    /// Total documents in index
    pub index_doc_count: usize,
    /// Whether this appears to be a retry (same session, different query within 5s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appears_retry: Option<bool>,
}

/// Explicit feedback from user/agent about discovery results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryFeedback {
    /// Timestamp of the feedback
    pub timestamp: DateTime<Utc>,
    /// Session ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// The query that was run
    pub query: String,
    /// Tool the user expected to find
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_tool: Option<String>,
    /// Actual results returned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_results: Option<Vec<String>>,
    /// Whether the results were useful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub useful: Option<bool>,
    /// Free-form note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Global logger instance
static LOGGER: OnceLock<Mutex<DiscoveryLogger>> = OnceLock::new();

/// Get or initialize the global discovery logger
pub fn logger() -> &'static Mutex<DiscoveryLogger> {
    LOGGER.get_or_init(|| Mutex::new(DiscoveryLogger::new()))
}

/// Discovery logger that writes to JSON-lines files
pub struct DiscoveryLogger {
    /// Path to the log directory
    log_dir: Option<PathBuf>,
    /// Cached writer for query log
    query_writer: Option<BufWriter<File>>,
    /// Cached writer for feedback log
    feedback_writer: Option<BufWriter<File>>,
    /// Last query info for retry detection
    last_query: Option<(String, DateTime<Utc>)>,
    /// Current session ID
    session_id: Option<String>,
    /// Whether logging is enabled
    enabled: bool,
}

impl DiscoveryLogger {
    /// Create a new discovery logger
    pub fn new() -> Self {
        let log_dir = Self::get_log_dir();
        let enabled = log_dir.is_some();

        Self {
            log_dir,
            query_writer: None,
            feedback_writer: None,
            last_query: None,
            session_id: None,
            enabled,
        }
    }

    /// Get the log directory, creating it if needed
    fn get_log_dir() -> Option<PathBuf> {
        // Check environment variable first
        if let Ok(dir) = std::env::var("JPX_DISCOVERY_LOG_DIR") {
            let path = PathBuf::from(dir);
            if std::fs::create_dir_all(&path).is_ok() {
                return Some(path);
            }
        }

        // Default to ~/.jpx/logs/
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".jpx").join("logs");
            if std::fs::create_dir_all(&path).is_ok() {
                return Some(path);
            }
        }

        None
    }

    /// Set the session ID for correlating queries
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Log a query_tools call
    pub fn log_query(&mut self, entry: DiscoveryLogEntry) {
        if !self.enabled {
            return;
        }

        // Detect retries (different query within 5 seconds)
        let mut entry = entry;
        if let Some((last_query, last_time)) = &self.last_query {
            let elapsed = entry.timestamp.signed_duration_since(*last_time);
            if elapsed.num_seconds() < 5 && &entry.query != last_query {
                entry.appears_retry = Some(true);
            }
        }
        self.last_query = Some((entry.query.clone(), entry.timestamp));

        // Add session ID
        entry.session_id = self.session_id.clone();

        // Write to log file
        if let Err(e) = self.write_query_entry(&entry) {
            tracing::warn!("Failed to write discovery log: {}", e);
        }
    }

    /// Log explicit feedback
    pub fn log_feedback(&mut self, mut feedback: DiscoveryFeedback) {
        if !self.enabled {
            return;
        }

        feedback.session_id = self.session_id.clone();

        if let Err(e) = self.write_feedback_entry(&feedback) {
            tracing::warn!("Failed to write feedback log: {}", e);
        }
    }

    /// Write a query log entry
    fn write_query_entry(&mut self, entry: &DiscoveryLogEntry) -> std::io::Result<()> {
        let writer = self.get_or_create_query_writer()?;
        serde_json::to_writer(&mut *writer, entry)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    /// Write a feedback log entry
    fn write_feedback_entry(&mut self, feedback: &DiscoveryFeedback) -> std::io::Result<()> {
        let writer = self.get_or_create_feedback_writer()?;
        serde_json::to_writer(&mut *writer, feedback)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }

    /// Get or create the query log writer
    fn get_or_create_query_writer(&mut self) -> std::io::Result<&mut BufWriter<File>> {
        if self.query_writer.is_none() {
            let path = self
                .log_dir
                .as_ref()
                .ok_or_else(|| std::io::Error::other("No log directory"))?
                .join("discovery.jsonl");

            let file = OpenOptions::new().create(true).append(true).open(path)?;

            self.query_writer = Some(BufWriter::new(file));
        }

        Ok(self.query_writer.as_mut().unwrap())
    }

    /// Get or create the feedback log writer
    fn get_or_create_feedback_writer(&mut self) -> std::io::Result<&mut BufWriter<File>> {
        if self.feedback_writer.is_none() {
            let path = self
                .log_dir
                .as_ref()
                .ok_or_else(|| std::io::Error::other("No log directory"))?
                .join("discovery_feedback.jsonl");

            let file = OpenOptions::new().create(true).append(true).open(path)?;

            self.feedback_writer = Some(BufWriter::new(file));
        }

        Ok(self.feedback_writer.as_mut().unwrap())
    }
}

impl Default for DiscoveryLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_entry_serialization() {
        let entry = DiscoveryLogEntry {
            timestamp: Utc::now(),
            session_id: Some("test-session".to_string()),
            query: "backup".to_string(),
            top_k: 5,
            results: vec!["create_backup".to_string(), "restore_backup".to_string()],
            result_count: 2,
            query_duration_ms: 5,
            index_doc_count: 47,
            appears_retry: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("backup"));
        assert!(json.contains("create_backup"));
    }

    #[test]
    fn test_feedback_serialization() {
        let feedback = DiscoveryFeedback {
            timestamp: Utc::now(),
            session_id: None,
            query: "snapshot".to_string(),
            expected_tool: Some("create_backup".to_string()),
            actual_results: Some(vec![]),
            useful: Some(false),
            note: Some("Expected 'snapshot' to find backup tools".to_string()),
        };

        let json = serde_json::to_string(&feedback).unwrap();
        assert!(json.contains("snapshot"));
        assert!(json.contains("create_backup"));
    }

    #[test]
    fn test_logger_with_custom_dir() {
        let temp_dir = TempDir::new().unwrap();
        // SAFETY: This test runs in isolation and doesn't spawn threads that read this env var
        unsafe {
            std::env::set_var("JPX_DISCOVERY_LOG_DIR", temp_dir.path());
        }

        let mut logger = DiscoveryLogger::new();
        assert!(logger.is_enabled());

        let entry = DiscoveryLogEntry {
            timestamp: Utc::now(),
            session_id: None,
            query: "test".to_string(),
            top_k: 5,
            results: vec![],
            result_count: 0,
            query_duration_ms: 1,
            index_doc_count: 10,
            appears_retry: None,
        };

        logger.log_query(entry);

        // Check file was created
        let log_path = temp_dir.path().join("discovery.jsonl");
        assert!(log_path.exists());

        // SAFETY: This test runs in isolation and doesn't spawn threads that read this env var
        unsafe {
            std::env::remove_var("JPX_DISCOVERY_LOG_DIR");
        }
    }
}
