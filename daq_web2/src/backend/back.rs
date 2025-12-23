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
        let content_type = field.content_type().unwrap_or("<none>").to_string();
        let bytes = field.bytes().await.expect("Failed to read field bytes");

        println!(
            "Field: name='{}', file_name='{}', content_type='{}', size={}",
            name,
            file_name,
            content_type,
            bytes.len()
        );
    }
    println!("Processed {} fields in this upload.", count);

    Ok(())
}
