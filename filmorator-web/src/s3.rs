use aws_sdk_s3::{presigning::PresigningConfig, Client};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum ImageTier {
    Thumb,
    Preview,
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

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn from_env(bucket: String) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Self { client, bucket }
    }

    pub async fn with_endpoint(bucket: String, endpoint: &str) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(endpoint)
            .force_path_style(true)
            .build();
        let client = Client::from_conf(s3_config);
        Self { client, bucket }
    }

    pub async fn presign_url(
        &self,
        tier: ImageTier,
        filename: &str,
        expires_in: Duration,
    ) -> anyhow::Result<String> {
        let key = format!("{}/{filename}", tier.as_prefix());
        let presigning_config = PresigningConfig::builder().expires_in(expires_in).build()?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await?;
        Ok(presigned.uri().to_string())
    }

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
