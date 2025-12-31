use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
use crate::backend;

fn upload_id_to_folder_name(upload_id: Option<i64>) -> String {
    format!("{:03}", upload_id.expect("Upload ID should be set here"))
}
#[derive(Default)]
struct UploadLogsFormMetadata {
    // This field order matches the form field order in the HTML
    upload_name: Option<String>,
    start_time: Option<chrono::NaiveDateTime>,
    upload_time: chrono::NaiveDateTime,
    commit_hash: Option<String>,
    short_comments: Option<String>,
    long_notes: Option<String>,
}

#[server]
#[post("/upload_logs")]
pub async fn upload_logs(mut form: dioxus_fullstack::MultipartFormData) -> Result<()> {
    tracing::info!("Received upload_logs request");

    let mut upload_form = UploadLogsFormMetadata::default();
    upload_form.upload_time = chrono::Utc::now().naive_utc();

    let mut upload_id: Option<i64> = None;

    while let Ok(Some(mut field)) = form.next_field().await {
        let name = field.name().unwrap_or("<none>").to_string();

        if name.is_empty() {
            tracing::error!("Field with empty name encountered");
            return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                "Field with empty name encountered"
            ))));
        } else if name == "<none>" {
            tracing::error!("Field with missing name encountered");
            return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                "Field with missing name encountered"
            ))));
        } else if name == "upload_name" {
            upload_form.upload_name = Some(field.text().await?);
        } else if name == "start_time" {
            upload_form.start_time = Some(chrono::NaiveDateTime::parse_from_str(
                &field.text().await?,
                "%Y-%m-%dT%H:%M",
            )?);
        } else if name == "commit_hash" {
            upload_form.commit_hash = Some(field.text().await?);
        } else if name == "short_comments" {
            upload_form.short_comments = Some(field.text().await?);
        } else if name == "long_notes" {
            upload_form.long_notes = Some(field.text().await?);
        } else if name == "vcan_dbc_file" {
            // Check if we missed a field
            if upload_form.upload_name.is_none()
                || upload_form.start_time.is_none()
                || upload_form.commit_hash.is_none()
                || upload_form.short_comments.is_none()
                || upload_form.long_notes.is_none()
            {
                tracing::error!("Received vcan_dbc_file before required text fields");
                return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                    "Received vcan_dbc_file before required text fields"
                ))));
            }

            // We know we didn't miss a field, so we can continue
            let db = backend::db::get_db_pool().await;
            let formated_start_time = upload_form
                .start_time
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string();
            let formated_upload_time = upload_form
                .upload_time
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string();
            let query = "INSERT INTO Uploads (upload_name, commit_hash, start_time, upload_time, short_comments, long_notes, upload_status) VALUES (?, ?, ?, ?, ?, ?, 'in_progress')";
            let result = sqlx::query(query)
                .bind(upload_form.upload_name.clone().unwrap())
                .bind(upload_form.commit_hash.clone().unwrap())
                .bind(formated_start_time)
                .bind(formated_upload_time)
                .bind(upload_form.short_comments.clone().unwrap())
                .bind(upload_form.long_notes.clone().unwrap())
                .execute(db)
                .await?;
            upload_id = Some(result.last_insert_rowid());
            tracing::info!(
                "Created upload with ID {}",
                upload_id.expect("Upload ID should be set here")
            );
            backend::log::insert_log(
                upload_id.expect("Upload ID should be set here"),
                backend::log::LogLevel::Info,
                &format!(
                    "Created new upload entry: {}",
                    upload_id.expect("Upload ID should be set here")
                ),
            )
            .await?;

            let folder_path =
                std::path::Path::new(config::RAW_FOLDER).join(upload_id_to_folder_name(upload_id));
            if folder_path.exists() {
                tracing::error!(
                    "Upload folder already exists for upload ID {}",
                    upload_id.expect("Upload ID should be set here")
                );
                return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                    "Upload folder already exists for upload ID {}",
                    upload_id.expect("Upload ID should be set here")
                ))));
            }
            std::fs::create_dir_all(folder_path.as_path())?;
            let file_name_raw = field.file_name().unwrap_or("vcan_file.dbc");
            let file_name = std::path::Path::new(file_name_raw)
                .file_name()
                .expect("Failed to get VCAN DBC file name")
                .to_string_lossy();
            let file_path = folder_path.join(file_name.as_ref());
            let mut file = tokio::fs::File::create(&file_path).await?;
            while let Some(chunk) = field.chunk().await? {
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            }
        } else if name == "log_files" {
            // Process log files similarly to vcan_dbc_file
            let folder_path =
                std::path::Path::new(config::RAW_FOLDER).join(upload_id_to_folder_name(upload_id));
            let file_name_raw = field.file_name().unwrap_or("log_file.log");
            let file_name = std::path::Path::new(file_name_raw)
                .file_name()
                .expect("Failed to get log file name")
                .to_string_lossy();
            let file_path = folder_path.join(file_name.as_ref());
            if file_path.exists() {
                tracing::error!(
                    "Log file {} already exists for upload ID {}",
                    file_name,
                );
                return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                    "Log file {} already exists for upload ID {}",
                    file_name,
                    upload_id.expect("Upload ID should be set here")
                ))));
            }
            let mut file = tokio::fs::File::create(&file_path).await?;
            while let Some(chunk) = field.chunk().await? {
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            }
        } else {
            tracing::error!("Unknown field name encountered: {}", name);
            return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                "Unknown field name encountered: {}",
                name
            ))));
        }
    }

    tracing::info!(
        "Completed download for upload ID {}",
        upload_id.expect("Upload ID should be set here")
    );

    let folder_path = std::path::Path::new(config::RAW_FOLDER)
        .join(upload_id_to_folder_name(upload_id));
    let file_path = folder_path.join(config::META_FILE);
    let mut file = tokio::fs::File::create(&file_path).await?;
    let metadata = serde_json::json!({
        "upload_id": upload_id.expect("Upload ID should be set here"),
        "upload_name": upload_form.upload_name,
        "start_time": upload_form.start_time.map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
        "upload_time": upload_form.upload_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        "commit_hash": upload_form.commit_hash,
        "short_comments": upload_form.short_comments,
        "long_notes": upload_form.long_notes,
    });
    let metadata_string = serde_json::to_string_pretty(&metadata)?;
    tokio::io::AsyncWriteExt::write_all(&mut file, metadata_string.as_bytes()).await?;

    Ok(())
}
