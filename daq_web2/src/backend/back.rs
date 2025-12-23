use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::s_helpers;

#[server]
pub async fn create_test(input: String) -> Result<String, ServerFnError> {
    let db = s_helpers::db::get_db_pool().await;
    Ok(format!("Created test with input: {}", input))
}

#[server]
#[post("/upload_logs")]
pub async fn upload_logs(mut form: dioxus_fullstack::MultipartFormData) -> Result<()> {
    println!("Processing uploaded logs...");
    let mut count = 0;

    while let Ok(Some(field)) = form.next_field().await {
        count += 1;
        let name = field.name().unwrap_or("<none>").to_string();
        let file_name = field.file_name().unwrap_or("<none>").to_string();
        // let content_type = field.content_type().unwrap_or("<none>").to_string();
        let bytes = field.bytes().await.expect("Failed to read field bytes");

        // println!(
        //     "Field: name='{}', file_name='{}', content_type='{}', size={}",
        //     name,
        //     file_name,
        //     content_type,
        //     bytes.len()
        // );

        if name == "upload_name" {
            let upload_name = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            println!("Upload name: {}", upload_name);
        } else if name == "start_time" {
            let start_time = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            println!("Start time: {}", start_time);
        } else if name == "vcan_dbc_file" {
            // Process the DBC file
            println!("Processing DBC file: {}", file_name);
        } else if name == "log_files" {
            // Process the log files
            println!("Processing log file: {}", file_name);
        } else if name == "short_comments" {
            let comments = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            println!("Short comments: {}", comments);
        } else if name == "long_notes" {
            let notes = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            println!("Long notes: {}", notes);
        }
    }
    println!("Processed {} fields in this upload.", count);

    Ok(())
}
