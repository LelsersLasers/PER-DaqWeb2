use dioxus::prelude::*;

use crate::backend;

enum UploadResult {
    Success,
    Failure(String),
}

#[component]
#[allow(non_snake_case)]
pub fn Upload() -> Element {
    let mut upload_result = use_signal(|| None::<UploadResult>);
    let mut uploading = use_signal(|| false);

    rsx! {
        h1 { "Upload Page" }

        if *uploading.read() {
            dialog {
                class: "fixed top-0 left-0 w-full h-full bg-black bg-opacity-50 flex items-center justify-center",
                open: true,
                div { class: "bg-white p-6 rounded shadow-md text-center",
                    h2 { "Uploading..." }
                    p { "Please wait while your files are being uploaded. Do not close this tab." }
                }
            }
        }

        match upload_result.read().as_ref() {
            Some(UploadResult::Success) => rsx! {
                dialog {
                    class: "fixed top-0 left-0 w-full h-full bg-green-200 bg-opacity-50 flex items-center justify-center",
                    open: true,
                    div { class: "bg-white p-6 rounded shadow-md text-center",
                        h2 { "Upload Successful" }
                        button {
                            onclick: move |_| {
                                upload_result.set(None);
                            },
                            "Close"
                        }
                    }
                }
            },
            Some(UploadResult::Failure(err_msg)) => rsx! {
                dialog {
                    class: "fixed top-0 left-0 w-full h-full bg-red-200 bg-opacity-50 flex items-center justify-center",
                    open: true,
                    div { class: "bg-white p-6 rounded shadow-md text-center",
                        h2 { "Upload Failed" }
                        pre { "{err_msg}" }
                        button {
                            onclick: move |_| {
                                upload_result.set(None);
                            },
                            "Close"
                        }
                    }
                }
            },
            None => rsx! {},
        }

        form {
            class: "p-4 bg-cyan-500",
            method: "post",
            enctype: "multipart/form-data",
            onsubmit: move |e| async move {
                e.prevent_default();

                uploading.set(true);

                let upload_status = backend::back::upload_logs(e.into()).await;
                match upload_status {
                    Ok(_) => upload_result.set(Some(UploadResult::Success)),
                    Err(err) => upload_result.set(Some(UploadResult::Failure(err.to_string()))),
                }

                uploading.set(false);
            },

            // upload_logs relies on field order in this form. The files must come last.
            label { r#for: "upload_name", "Upload name:" }
            input {
                r#type: "text",
                id: "upload_name",
                name: "upload_name",
                required: true,
            }
            br {}

            label { r#for: "start_time", "Start time:" }
            input {
                r#type: "datetime-local",
                id: "start_time",
                name: "start_time",
                required: true,
            }
            br {}

            label { r#for: "commit_hash", "firmware commit hash:" }
            input {
                r#type: "text",
                required: true,
                id: "commit_hash",
                name: "commit_hash",
            }
            br {}

            // Not required = can be blank ("") not that the field is missing
            label { r#for: "short_comments", "Short comments (optional):" }
            input {
                r#type: "text",
                id: "short_comments",
                name: "short_comments",
            }
            br {}
            // Not required = can be blank ("") not that the field is missing
            label { r#for: "long_notes", "Long notes (optional):" }
            br {}
            textarea {
                id: "long_notes",
                name: "long_notes",
                rows: "10",
                cols: "50",
            }
            br {}

            // TAGS
            label { r#for: "vcan_dbc_file", "VCAN DBC file:" }
            input {
                r#type: "file",
                id: "vcan_dbc_file",
                name: "vcan_dbc_file",
                accept: ".dbc",
                required: true,
            }
            br {}

            label { r#for: "log_files", "Log raw files folder:" }
            input {
                r#type: "file",
                id: "log_files",
                name: "log_files",
                directory: true,
                required: true,
            }
            br {}

            button { r#type: "submit", "Upload" }
        }
    }
}
