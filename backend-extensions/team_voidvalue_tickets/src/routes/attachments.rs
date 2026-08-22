use crate::{
    repository::{self, AttachmentDownload, NewAttachment, TicketScope},
    settings::ExtensionSettingsData,
};
use aws_sdk_s3::{
    Client as S3Client,
    config::{BehaviorVersion, Config, Credentials, Region},
};
use axum::extract::Multipart;
use sha2::{Digest, Sha256};
use shared::{State, response::ApiResponse, settings::StorageDriver};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub async fn upload(
    state: &State,
    ticket_uuid: Uuid,
    uploader_uuid: Uuid,
    scope: TicketScope,
    mut multipart: Multipart,
) -> anyhow::Result<crate::models::TicketDetail> {
    let settings = state.settings.get().await?;
    let ext: &ExtensionSettingsData = settings.find_extension_settings()?;
    if !ext.attachments_enabled {
        anyhow::bail!("attachments are disabled");
    }
    let max_bytes = ext.attachment_max_bytes;
    let max_files = ext.attachment_max_files;
    let allowed_mime_types = ext.allowed_mime_types.clone();
    drop(settings);

    let mut message_uuid = None;
    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("message_uuid") => {
                message_uuid = Some(field.text().await?.parse::<Uuid>()?);
            }
            Some("files") => {
                if files.len() >= usize::try_from(max_files)? {
                    anyhow::bail!("too many attachments");
                }
                let filename = sanitize_filename(
                    field
                        .file_name()
                        .ok_or_else(|| anyhow::anyhow!("attachment filename is missing"))?,
                )?;
                let mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                if !allowed_mime_types
                    .iter()
                    .any(|allowed| allowed == &mime_type)
                {
                    anyhow::bail!("attachment type is not allowed");
                }
                let content = field.bytes().await?.to_vec();
                if u64::try_from(content.len())? > max_bytes {
                    anyhow::bail!("attachment is too large");
                }
                let digest = Sha256::digest(&content);
                let mut sha256 = String::with_capacity(64);
                const HEX: &[u8; 16] = b"0123456789abcdef";
                for byte in digest {
                    sha256.push(HEX[usize::from(byte >> 4)] as char);
                    sha256.push(HEX[usize::from(byte & 0x0f)] as char);
                }
                files.push(NewAttachment {
                    original_filename: filename,
                    mime_type,
                    content,
                    sha256,
                });
            }
            _ => {}
        }
    }
    if files.is_empty() {
        anyhow::bail!("no attachments provided");
    }
    repository::add_attachments(
        state,
        ticket_uuid,
        message_uuid.ok_or_else(|| anyhow::anyhow!("message uuid is missing"))?,
        uploader_uuid,
        scope,
        files,
        max_files,
    )
    .await
}

fn sanitize_filename(filename: &str) -> anyhow::Result<String> {
    let filename = filename
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|character| !character.is_control())
        .take(255)
        .collect::<String>();
    if filename.trim().is_empty() || filename == "." || filename == ".." {
        anyhow::bail!("attachment filename is invalid");
    }
    Ok(filename)
}

pub fn download_response(file: AttachmentDownload) -> ApiResponse {
    let safe_filename = file
        .original_filename
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    ApiResponse::new(axum::body::Body::from(file.content))
        .with_header("Content-Type", file.mime_type)
        .with_header(
            "Content-Disposition",
            format!("attachment; filename=\"{safe_filename}\""),
        )
        .with_header("X-Content-Type-Options", "nosniff")
}

pub async fn read_private_file(state: &State, path: &str) -> anyhow::Result<Vec<u8>> {
    if !path.starts_with("privatedata/extensions/team.voidvalue.tickets/")
        || path.contains("..")
        || path.starts_with('/')
    {
        anyhow::bail!("invalid attachment storage path");
    }
    let settings = state.settings.get().await?;
    match &settings.storage_driver {
        StorageDriver::Filesystem { .. } => {
            let filesystem = settings
                .storage_driver
                .get_cap_filesystem("")
                .await
                .ok_or_else(|| anyhow::anyhow!("filesystem storage unavailable"))??;
            let mut file = filesystem.async_open(path).await?;
            let mut content = Vec::new();
            file.read_to_end(&mut content).await?;
            Ok(content)
        }
        StorageDriver::S3 {
            access_key,
            secret_key,
            bucket,
            region,
            endpoint,
            path_style,
            ..
        } => {
            let config = Config::builder()
                .behavior_version(BehaviorVersion::latest())
                .credentials_provider(Credentials::new(
                    access_key.as_str(),
                    secret_key.as_str(),
                    None,
                    None,
                    "team-voidvalue-tickets",
                ))
                .region(Region::new(region.to_string()))
                .endpoint_url(endpoint.as_str())
                .force_path_style(*path_style)
                .build();
            let response = S3Client::from_conf(config)
                .get_object()
                .bucket(bucket.as_str())
                .key(path)
                .send()
                .await?;
            Ok(response.body.collect().await?.into_bytes().to_vec())
        }
    }
}
