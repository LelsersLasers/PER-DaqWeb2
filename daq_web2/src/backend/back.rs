use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
use crate::{backend, s_helpers};

#[server]
pub async fn create_test(input: String) -> Result<String, ServerFnError> {
    tracing::info!("in create_test with input: {}", input);
    let db = s_helpers::db::get_db_pool().await;
    Ok(format!("okay test with input: {}", input))
}

#[derive(Default)]
struct UploadLogsForm {
    // This field order matches the form field order in the HTML
    upload_name: Option<String>,
    start_time: Option<chrono::NaiveDateTime>,
    upload_time: chrono::NaiveDateTime,
    commit_hash: Option<String>,
    short_comments: Option<String>,
    long_notes: Option<String>,
    // vcan_dbc_file: Option<bytes::Bytes>,
    // log_files: Option<Vec<bytes::Bytes>>,
}

#[server]
#[post("/upload_logs")]
pub async fn upload_logs(mut form: dioxus_fullstack::MultipartFormData) -> Result<()> {
    tracing::info!("Received upload_logs request");

    let mut upload_form = UploadLogsForm::default();
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
            let db = s_helpers::db::get_db_pool().await;
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
                "Created new upload entry",
            )
            .await?;

            let folder_path = std::path::Path::new(config::RAW_FOLDER)
                .join(upload_id.expect("Upload ID should be set here").to_string());
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
                .unwrap_or(std::ffi::OsStr::new("vcan_file.dbc"))
                .to_string_lossy();
            let file_path = folder_path.join(file_name.as_ref());
            let mut file = tokio::fs::File::create(&file_path).await?;
            while let Some(chunk) = field.chunk().await? {
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            }
        } else if name == "log_files" {
            // Process log files similarly to vcan_dbc_file
            let upload_id = upload_id.expect("Upload ID should be set before log_files");
            let folder_path = std::path::Path::new(config::RAW_FOLDER).join(upload_id.to_string());
            let file_name_raw = field.file_name().unwrap_or("log_file.log");
            let file_name = std::path::Path::new(file_name_raw)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("log_file.log"))
                .to_string_lossy();
            let file_path = folder_path.join(file_name.as_ref());
            if file_path.exists() {
                tracing::error!(
                    "Log file {} already exists for upload ID {}",
                    file_name,
                    upload_id
                );
                return Err(dioxus::CapturedError(std::sync::Arc::new(anyhow::anyhow!(
                    "Log file {} already exists for upload ID {}",
                    file_name,
                    upload_id
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

    Ok(())
}
