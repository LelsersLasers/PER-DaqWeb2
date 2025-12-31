use dioxus::prelude::*;

use crate::config;

#[cfg(feature = "server")]
use crate::backend;

const CAN_EFF_FLAG: u32 = 0x80000000;
const CAN_EXT_ID_MASK: u32 = 0x1FFFFFFF;
const CAN_STD_ID_MASK: u32 = 0x000007FF;

const FRAME_TYPE_OFFSET: usize = 1;
const MSG_BYTE_LEN: usize = 19;
const TIMESTAMP_OFFSET: usize = 5;
const ID_OFFSET: usize = 9;
const DLC_OFFSET: usize = 10;
const DATA_OFFSET: usize = 11;

const BATCH_SIZE: usize = 100;

#[cfg(feature = "server")]
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
            let file_path = folder_path.join(config::DBC_FILE_NAME);
            let mut file = tokio::fs::File::create(&file_path).await?;
            while let Some(chunk) = field.chunk().await? {
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            }
            tokio::fs::File::sync_all(&file).await?;
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
                    upload_id.expect("Upload ID should be set here")
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
            tokio::fs::File::sync_all(&file).await?;
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

    let folder_path =
        std::path::Path::new(config::RAW_FOLDER).join(upload_id_to_folder_name(upload_id));
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
    tokio::fs::File::sync_all(&file).await?;

    tokio::spawn(async move {
        process_uploaded_logs(
            upload_id.expect("Upload ID should be set here"),
            upload_form
                .start_time
                .expect("Start time should be set here"),
        )
        .await;
    });

    Ok(())
}

#[cfg(feature = "server")]
async fn process_uploaded_logs(upload_id: i64, start_time: chrono::NaiveDateTime) {
    tracing::info!("Starting processing for upload ID {}", upload_id);

    let db = backend::db::get_db_pool().await;
    let folder_path =
        std::path::Path::new(config::RAW_FOLDER).join(upload_id_to_folder_name(Some(upload_id)));
    let dbc_file = folder_path.join(config::DBC_FILE_NAME);
    if !dbc_file.exists() {
        let error_message = format!(
            "DBC file not found for upload ID {}: expected at {}",
            upload_id,
            dbc_file.display()
        );
        tracing::error!("{}", error_message);
        let _ = backend::log::insert_log(upload_id, backend::log::LogLevel::Error, &error_message)
            .await
            .expect("Failed to log DBC file missing error");
        return;
    }

    let parser = match can_decode::Parser::from_dbc_file(&dbc_file) {
        Ok(p) => p,
        Err(e) => {
            let error_message = format!(
                "Failed to create DBC parser for upload ID {}: {}",
                upload_id, e
            );
            tracing::error!("{}", error_message);
            let _ =
                backend::log::insert_log(upload_id, backend::log::LogLevel::Error, &error_message)
                    .await
                    .expect("Failed to log DBC parser creation error");
            return;
        }
    };

    let mut read_paths = tokio::fs::read_dir(&folder_path)
        .await
        .expect("Failed to read upload folder");
    let mut log_file_paths = vec![];
    while let Some(entry) = read_paths
        .next_entry()
        .await
        .expect("Failed to read directory entry")
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
            log_file_paths.push(path);
        }
    }
    log_file_paths.sort();

    for log_file in log_file_paths {
        let file_name = log_file
            .file_name()
            .and_then(|s| s.to_str())
            .expect("Failed to get log file name");
        tracing::info!(
            "Processing log file {} for upload ID {}",
            file_name,
            upload_id
        );
        parse_log_file(upload_id, &log_file, &parser, db, start_time).await;
    }

    tracing::info!("Completed processing for upload ID {}", upload_id);

    let query = "UPDATE Uploads SET upload_status = 'completed' WHERE id = ?";
    sqlx::query(query)
        .bind(upload_id)
        .execute(db)
        .await
        .expect("Failed to update upload status to completed");
    let _ = backend::log::insert_log(
        upload_id,
        backend::log::LogLevel::Info,
        "Upload processing completed successfully",
    );
}

#[cfg(feature = "server")]
struct ParsedMessage {
    timestamp_raw: u32,
    timestamp_adj: chrono::NaiveDateTime,
    decoded: can_decode::DecodedMessage,
}
#[cfg(feature = "server")]
async fn parse_log_file(
    upload_id: i64,
    in_file: &std::path::Path,
    parser: &can_decode::Parser,
    db: &sqlx::SqlitePool,
    start_time: chrono::NaiveDateTime,
) {
    let content = tokio::fs::read(in_file)
        .await
        .expect("Failed to read log file");

    let query = "INSERT INTO Logs (file_name, upload_id) VALUES (?, ?)";
    let result = sqlx::query(query)
        .bind(in_file.file_name().unwrap().to_string_lossy().to_string())
        .bind(upload_id)
        .execute(db)
        .await
        .expect("Failed to insert log file entry");
    let log_id = result.last_insert_rowid();

    let mut parsed = Vec::with_capacity(BATCH_SIZE);

    let mut offset = 0;
    while offset + MSG_BYTE_LEN <= content.len() {
        let timestamp = u32::from_le_bytes(
            content[offset + FRAME_TYPE_OFFSET..offset + TIMESTAMP_OFFSET]
                .try_into()
                .unwrap(),
        );
        let can_id = u32::from_le_bytes(
            content[offset + TIMESTAMP_OFFSET..offset + ID_OFFSET]
                .try_into()
                .unwrap(),
        );
        // let _bus_id = content[offset + consts::ID_OFFSET];
        let dlc = content[offset + DLC_OFFSET];
        let data = &content[offset + DATA_OFFSET..offset + DATA_OFFSET + dlc as usize];
        offset += MSG_BYTE_LEN;

        let is_extended = (can_id & CAN_EFF_FLAG) != 0;
        let arb_id = if is_extended {
            can_id & CAN_EXT_ID_MASK
        } else {
            can_id & CAN_STD_ID_MASK
        };

        match parser.decode_msg(arb_id, data) {
            Some(decoded) => {
                parsed.push(ParsedMessage {
                    timestamp_raw: timestamp,
                    timestamp_adj: start_time + chrono::Duration::milliseconds(timestamp as i64),
                    // TODO: fix start time logic
                    decoded,
                });
                if parsed.len() >= BATCH_SIZE {
                    if let Err(e) = insert_msg_batch(db, log_id, &parsed).await {
                        let error_message = format!(
                            "{} - {}: Failed to insert message batch: {}",
                            upload_id,
                            in_file.display(),
                            e
                        );
                        tracing::error!("{}", error_message);
                        let _ = backend::log::insert_log(
                            upload_id,
                            backend::log::LogLevel::Error,
                            &error_message,
                        )
                        .await
                        .expect("Failed to log message batch insertion error");
                    }
                    parsed.clear();
                }
            }
            None => {
                let error_message = format!(
                    "{} - {}: Failed to decode message with ID {:X} at timestamp {}",
                    upload_id,
                    in_file.display(),
                    arb_id,
                    timestamp
                );
                tracing::warn!("{}", error_message);
                let _ = backend::log::insert_log(
                    upload_id,
                    backend::log::LogLevel::Warning,
                    &error_message,
                )
                .await
                .expect("Failed to log message decoding warning");
            }
        }
    }

    if !parsed.is_empty() {
        if let Err(e) = insert_msg_batch(db, log_id, &parsed).await {
            let error_message = format!(
                "{} - {}: Failed to insert final message batch: {}",
                upload_id,
                in_file.display(),
                e
            );
            tracing::error!("{}", error_message);
            let _ =
                backend::log::insert_log(upload_id, backend::log::LogLevel::Error, &error_message)
                    .await
                    .expect("Failed to log final message batch insertion error");
        }
    }

    let extra_bytes = content.len() - offset;
    if extra_bytes > 0 {
        let warning_message = format!(
            "{} - {}: {} extra bytes at end of file",
            upload_id,
            in_file.display(),
            extra_bytes
        );
        tracing::warn!("{}", warning_message);
        let _ =
            backend::log::insert_log(upload_id, backend::log::LogLevel::Warning, &warning_message)
                .await
                .expect("Failed to log extra bytes warning");
    }
}

#[cfg(feature = "server")]
async fn insert_msg_batch(
    db: &sqlx::SqlitePool,
    log_id: i64,
    batch: &[ParsedMessage],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    for parsed_msg in batch {
        let query = "INSERT INTO Messages (log_id, arbitration_id, msg_name, timestamp_raw, timestamp_adj) VALUES (?, ?, ?, ?, ?)";
        let result = sqlx::query(query)
            .bind(log_id)
            .bind(parsed_msg.decoded.msg_id as i64)
            .bind(parsed_msg.decoded.name.clone())
            .bind(parsed_msg.timestamp_raw as i64)
            .bind(
                parsed_msg
                    .timestamp_adj
                    .format("%Y-%m-%d %H:%M:%S%.3f")
                    .to_string(),
            )
            .execute(&mut *tx)
            .await?;
        let msg_id = result.last_insert_rowid();

        for (signal_name, signal_value) in parsed_msg.decoded.signals.iter() {
            let query = "INSERT INTO Signals (msg_id, signal_name, signal_value, signal_unit) VALUES (?, ?, ?, ?)";
            sqlx::query(query)
                .bind(msg_id)
                .bind(signal_name.clone())
                .bind(signal_value.value)
                .bind(signal_value.unit.clone())
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
