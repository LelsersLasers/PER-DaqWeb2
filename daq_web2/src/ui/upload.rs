use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Upload() -> Element {
    rsx! {
        h1 { "Upload Page" }

        form {
            class: "p-4 bg-cyan-500",
            
            label { r#for: "upload_name", "Upload name:" }
            input { r#type: "text", id: "upload_name", name: "upload_name", required: true }
            br {}

            label { r#for: "start_time", "Start time:" }
            input { r#type: "datetime-local", id: "start_time", name: "start_time", required: true }
            br {}

            // TAGS

            label { r#for: "vcan_dbc_file", "VCAN DBC file:" }
            input { r#type: "file", id: "vcan_dbc_file", name: "vcan_dbc_file", accept: ".dbc", required: true }
            br {}

            label { r#for: "log_files", "Log raw files folder:" }
            input { r#type: "file", id: "log_files", name: "log_files", directory: true, required: true }
            br {}

            label { r#for: "short_comments", "Short comments (optional):" }
            input { r#type: "text", id: "short_comments", name: "short_comments" }
            br {}

            label { r#for: "long_notes", "Long notes (optional):" }
            br {}
            textarea { id: "long_notes", name: "long_notes", rows: "10", cols: "50" }
            br {}


            // button { "Upload" }
        }
    }
}
