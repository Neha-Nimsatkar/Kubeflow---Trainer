// Copyright The Kubeflow Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use iceberg::io::S3_PATH_STYLE_ACCESS;

/// Cache index column name used across the data cache system
pub const CACHE_INDEX_COLUMN: &str = "cache_index";

/// Index pair representing a start and end range
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct IndexPair {
    pub start: u64,
    pub end: u64,
}

/// Configuration for dataset metadata and table information
#[derive(Debug, Clone)]
pub struct DatasetConfig {
    pub metadata_loc: String,
    pub schema_name: String,
    pub table_name: String,
}

/// Comprehensive configuration for the data cache system
/// Consolidates all environment variables used across the application
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub dataset: DatasetConfig,
    pub connect_timeout: Duration,
}

impl DatasetConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata_loc = env::var("METADATA_LOC")?;
        let schema_name = env::var("SCHEMA_NAME")?;
        let table_name = env::var("TABLE_NAME")?;
        Ok(DatasetConfig {
            metadata_loc,
            schema_name,
            table_name,
        })
    }
}

impl CacheConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let dataset = DatasetConfig::from_env()?;

        // Connection timeout with default of 20 seconds
        let connect_timeout = env::var("CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(20));

        Ok(CacheConfig {
            dataset,
            connect_timeout,
        })
    }

    /// Create shared configuration from environment variables
    /// Returns Arc<CacheConfig> for efficient sharing across components
    pub fn shared_from_env() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        Ok(Arc::new(Self::from_env()?))
    }

    /// Create a new configuration with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }
}

/// Storage properties for Iceberg file access. Path-style addressing
/// (`endpoint/bucket/key`) is needed for SeaweedFS and MinIO. The default
/// stays virtual-hosted style, so AWS users are not affected.
pub fn s3_props(path_style: Option<String>) -> Vec<(String, String)> {
    let mut props = Vec::new();
    if let Some(v) = path_style {
        props.push((S3_PATH_STYLE_ACCESS.to_string(), v));
    }
    props
}

/// Reads `S3_PATH_STYLE_ACCESS` (for example `true`) from the environment.
pub fn s3_props_from_env() -> Vec<(String, String)> {
    s3_props(env::var("S3_PATH_STYLE_ACCESS").ok())
}

#[cfg(test)]
mod s3_props_tests {
    use super::s3_props;

    #[test]
    fn no_props_by_default() {
        assert!(s3_props(None).is_empty());
    }

    #[test]
    fn path_style_is_passed_through() {
        assert_eq!(
            s3_props(Some("true".to_string())),
            vec![("s3.path-style-access".to_string(), "true".to_string())]
        );
    }
}