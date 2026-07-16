//! Download manager for HuggingFace model retrieval.
//!
//! Supports parallel downloads, resume, progress tracking, retry,
//! SHA-256 verification, and HF_TOKEN authentication.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

/// Configuration for a model download.
#[derive(Debug, Clone)]
pub struct DownloadSpec {
    /// HuggingFace repository (e.g., "Qwen/Qwen3-8B-GGUF").
    pub repo: String,
    /// Filename within the repository.
    pub filename: String,
    /// Local destination path.
    pub dest: PathBuf,
    /// Expected SHA-256 checksum (optional).
    pub sha256: Option<String>,
    /// Expected file size in bytes (optional).
    pub size: Option<u64>,
}

/// The download manager handles all model retrieval from HuggingFace.
pub struct DownloadManager {
    /// HTTP client.
    client: Client,
    /// Base URL for HuggingFace downloads.
    base_url: String,
    /// HF_TOKEN for authentication.
    token: Option<String>,
    /// Maximum concurrent downloads.
    max_concurrent: usize,
    /// Maximum retry attempts.
    max_retries: u32,
}

impl DownloadManager {
    /// Create a new download manager.
    pub fn new(max_concurrent: usize) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .build()
            .context("failed to build HTTP client")?;

        let token = std::env::var("HF_TOKEN").ok();

        Ok(Self {
            client,
            base_url: "https://huggingface.co".to_string(),
            token,
            max_concurrent,
            max_retries: 3,
        })
    }

    /// Set a custom mirror URL.
    pub fn with_mirror(mut self, mirror: &str) -> Self {
        self.base_url = mirror.to_string();
        self
    }

    /// Download a model file with progress tracking.
    pub async fn download(&self, spec: &DownloadSpec) -> Result<()> {
        // Create destination directory
        if let Some(parent) = spec.dest.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        // Skip if file already exists
        if spec.dest.exists() {
            info!("model already downloaded: {}", spec.dest.display());

            // Verify checksum if provided
            if let Some(ref expected) = spec.sha256 {
                if !self.verify_checksum(&spec.dest, expected).await? {
                    warn!("checksum mismatch, re-downloading: {}", spec.dest.display());
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }

        let url = format!(
            "{}/{}/resolve/main/{}",
            self.base_url, spec.repo, spec.filename
        );

        info!("downloading: {} -> {}", url, spec.dest.display());

        let mut attempt = 0;
        loop {
            match self.download_with_resume(&url, &spec.dest, spec.size).await {
                Ok(()) => break,
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.max_retries {
                        return Err(e).context("download failed after max retries");
                    }
                    warn!(
                        "download attempt {} failed: {}, retrying...",
                        attempt, e
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                }
            }
        }

        // Verify checksum
        if let Some(ref expected) = spec.sha256 {
            if !self.verify_checksum(&spec.dest, expected).await? {
                fs::remove_file(&spec.dest).await.ok();
                anyhow::bail!("checksum verification failed for {}", spec.dest.display());
            }
        }

        info!("download complete: {}", spec.dest.display());
        Ok(())
    }

    /// Download with resume support.
    async fn download_with_resume(
        &self,
        url: &str,
        dest: &Path,
        total_size: Option<u64>,
    ) -> Result<()> {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add auth token if available
        if let Some(ref token) = self.token {
            headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        // Check for partial download
        let existing_size = if dest.exists() {
            fs::metadata(dest).await.map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        if existing_size > 0 {
            headers.insert(
                "Range",
                format!("bytes={}-", existing_size).parse().unwrap(),
            );
        }

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await
            .context("failed to send download request")?;

        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
            anyhow::bail!("download request failed with status: {}", status);
        }

        let content_length = response.content_length().unwrap_or(0);
        let total = total_size.unwrap_or(existing_size + content_length);

        // Create progress bar
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("invalid progress template"),
        );
        pb.set_position(existing_size);

        // Open file for writing (append if resuming)
        let mut file = if existing_size > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
            fs::OpenOptions::new()
                .append(true)
                .open(dest)
                .await
                .context("failed to open file for resume")?
        } else {
            fs::File::create(dest)
                .await
                .context("failed to create download file")?
        };

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("failed to read download chunk")?;
            file.write_all(&chunk)
                .await
                .context("failed to write download chunk")?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("download complete");
        Ok(())
    }

    /// Verify SHA-256 checksum of a file.
    pub async fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool> {
        let data = fs::read(path)
            .await
            .with_context(|| format!("failed to read file for checksum: {}", path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        let hex = hex::encode(result);

        Ok(hex == expected)
    }

    /// Download multiple files in parallel.
    pub async fn download_all(&self, specs: &[DownloadSpec]) -> Result<()> {
        use tokio::sync::Semaphore;

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for spec in specs {
            let permit = semaphore.clone().acquire_owned().await?;
            let spec = spec.clone();
            let manager = Self {
                client: self.client.clone(),
                base_url: self.base_url.clone(),
                token: self.token.clone(),
                max_concurrent: 1,
                max_retries: self.max_retries,
            };

            let handle = tokio::spawn(async move {
                let result = manager.download(&spec).await;
                drop(permit);
                result
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.context("download task panicked")??;
        }

        Ok(())
    }
}
