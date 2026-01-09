use aws_sdk_s3::{presigning::PresigningConfig, Client};
use std::time::Duration;

/// Image size tiers for progressive loading.
#[derive(Debug, Clone, Copy)]
pub enum ImageTier {
    /// Small thumbnail (~200px)
    Thumb,
    /// Medium preview (~800px)
    Preview,
    /// Full resolution original
    Original,
}

impl ImageTier {
    #[must_use]
    pub fn as_prefix(self) -> &'static str {
        match self {
            Self::Thumb => "thumb",
            Self::Preview => "preview",
            Self::Original => "original",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "thumb" => Some(Self::Thumb),
            "preview" => Some(Self::Preview),
            "original" => Some(Self::Original),
            _ => None,
        }
    }
}

/// S3 client wrapper for photo storage.
#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    /// Creates a new S3 client from environment configuration.
    pub async fn from_env(bucket: String) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Self { client, bucket }
    }

    /// Creates a new S3 client with custom endpoint (for MinIO/local dev).
    pub async fn with_endpoint(bucket: String, endpoint: &str) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        let client = Client::from_conf(s3_config);
        Self { client, bucket }
    }

    /// Generates a presigned URL for an image.
    pub async fn presign_url(
        &self,
        tier: ImageTier,
        filename: &str,
        expires_in: Duration,
    ) -> Result<
        String,
        aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    > {
        let key = format!("{}/{filename}", tier.as_prefix());

        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .expect("valid presigning config");

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Lists all photos in the original tier.
    pub async fn list_photos(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut filenames = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix("original/");

            if let Some(token) = &continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        if let Some(filename) = key.strip_prefix("original/") {
                            if !filename.is_empty() {
                                filenames.push(filename.to_string());
                            }
                        }
                    }
                }
            }

            if response.is_truncated == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        filenames.sort();
        Ok(filenames)
    }
}
