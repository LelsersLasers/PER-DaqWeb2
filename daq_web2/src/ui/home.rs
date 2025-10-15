use crate::assets;
use crate::backend;
use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
    }
}

#[component]
#[allow(non_snake_case)]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: assets::HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

#[derive(Clone)]
enum EchoMessage {
    OnInput(String),
}

struct EchoState {
    msg: String,
}

#[component]
#[allow(non_snake_case)]
fn Echo() -> Element {
    let mut echo_state = use_signal(|| EchoState { msg: String::new() });
    let mut echo_msgs = use_signal(Vec::<EchoMessage>::new);
    let mut new_msgs = use_signal(Vec::<EchoMessage>::new);

    // First effect: Process input messages, may queue new messages
    use_effect(move || {
        let mut new_messages: Vec<EchoMessage> = Vec::new();
        for echo_msg in echo_msgs.read().iter() {
            match echo_msg {
                EchoMessage::OnInput(data) => {
                    echo_state.write().msg = data.to_string();
                }
            }
        }
        if !new_messages.is_empty() {
            *new_msgs.write() = new_messages; // Write new messages to the separate signal
        }
    });

    // Second effect: Move new_msgs to echo_msgs for next loop
    use_effect(move || {
        if !new_msgs.read().is_empty() {
            *echo_msgs.write() = new_msgs.read().clone();
            new_msgs.write().clear();
        }
    });

    // Third effect: Clear processed input messages
    use_effect(move || {
        echo_msgs.write().clear();
    });

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    // let data = create_test(event.value()).await.unwrap();
                    let data = match backend::back::create_test(event.value().to_string()).await {
                        Ok(res) => res,
                        Err(err) => format!("Error: {}", err),
                    };
                    // response.set(data);
                    // echo_state.modify(|state| state.msg = data);
                    // echo_state.msg = data;
                    echo_msgs.push(EchoMessage::OnInput(data));
                },
            }

            // if !response().is_empty() {
            //     p {
            //         "Server echoed: "
            //         i { "{response}" }
            //     }
            // }

            if !echo_state.read().msg.is_empty() {
                p {
                    "Server echoed: "
                    i { "{echo_state.read().msg}" }
                }
            }
        }
    }
}
