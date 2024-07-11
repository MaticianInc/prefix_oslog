use std::{borrow::Borrow, cmp::Ordering, collections::BTreeMap, env, str::FromStr};

use crate::OsLog;
use dashmap::DashMap;
use log::{Level, LevelFilter, Log, Metadata, Record};

pub struct OsLogger {
    default_level: LevelFilter,
    filters: BTreeMap<Prefix, LevelFilter>,
    loggers: DashMap<String, OsLog>,
    subsystem: String,
    stderr_out: bool,
}

#[derive(PartialEq, Eq)]
struct Prefix(String);

impl PartialOrd for Prefix {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Prefix {
    fn cmp(&self, other: &Self) -> Ordering {
        let length_order = other.0.len().cmp(&self.0.len());
        match length_order {
            Ordering::Less | Ordering::Greater => length_order,
            Ordering::Equal => other.0.cmp(&self.0),
        }
    }
}

impl Borrow<str> for Prefix {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for Prefix {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl Log for OsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let level = self
            .filters
            .iter()
            .find_map(|(prefix, level)| metadata.target().strip_prefix(&prefix.0).map(|_| level))
            .unwrap_or(&self.default_level);
        metadata.level() <= *level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let target = record.metadata().target();
            let logger = self
                .loggers
                .entry(target.to_owned())
                .or_insert_with(|| OsLog::new(&self.subsystem, target));
            let message = std::format!("[{}] {}", target, record.args());
            logger.with_level(record.level().into(), &message);
            }
        }
    }

    fn flush(&self) {}
}

impl OsLogger {
    /// Creates a new logger. You must also call `init` to finalize the set up.
    /// By default the level filter will be set to `LevelFilter::Trace`.
    pub fn new(subsystem: &str, global_level: LevelFilter) -> Self {
        Self {
            default_level: global_level,
            filters: BTreeMap::new(),
            loggers: DashMap::new(),
            subsystem: subsystem.to_string(),
        }
    }

    pub fn level_filter(level: LevelFilter) {
        log::set_max_level(level);
    }

    /// Sets or updates the category's level filter.
    pub fn target_level_filter(mut self, prefix: &str, level: LevelFilter) -> Self {
        if let Some(filter) = self.filters.get_mut(prefix) {
            *filter = level;
        } else {
            self.filters.insert(prefix.into(), level);
        }

        self
    }

    pub fn init(self) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, trace, warn};

    #[test]
    fn test_basic_usage() {
        OsLogger::new("com.example.oslog")
            .level_filter(LevelFilter::Trace)
            .category_level_filter("Settings", LevelFilter::Warn)
            .category_level_filter("Database", LevelFilter::Error)
            .category_level_filter("Database", LevelFilter::Trace)
            .init()
            .unwrap();

        // This will not be logged because of its category's custom level filter.
        info!(target: "Settings", "Info");

        warn!(target: "Settings", "Warn");
        error!(target: "Settings", "Error");

        trace!("Trace");
        debug!("Debug");
        info!("Info");
        warn!(target: "Database", "Warn");
        error!("Error");
    }
}
