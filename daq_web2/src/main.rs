use dioxus::prelude::*;

mod assets;
mod config;

#[cfg(feature = "server")]
mod s_helpers;

#[cfg(feature = "web")]
mod ui;


#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(ui::navbar::Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    #[cfg(feature = "web")]
    // Hydrate the application on the client
    dioxus::launch(App);

    // Launch axum on the server
    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                launch_server(App).await;
            });
    }
}

#[cfg(feature = "server")]
async fn launch_server(component: fn() -> Element) {
    let ip =
        dioxus::cli_config::server_ip().unwrap_or_else(|| config::SERVER_ADDR.parse().unwrap());
    let port = dioxus::cli_config::server_port().unwrap_or(config::SERVER_PORT);
    tracing::info!("Listening on http://{ip}:{port}");

    let address = std::net::SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::default(), component)
        .into_make_service();
    axum::serve(listener, router).await.unwrap();
}

#[component]
#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: assets::FAVICON }
        document::Link {
            rel: "stylesheet", href: assets::MAIN_CSS
        }
        document::Link {
            rel: "stylesheet", href: assets::TAILWIND_CSS
        }
        Router::<Route> {}
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

/// Home page
#[component]
#[allow(non_snake_case)]
fn Home() -> Element {
    rsx! {
        Hero {}
        Echo {}
    }
}

/// Blog page
#[component]
#[allow(non_snake_case)]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.


/// Echo component that demonstrates fullstack server functions.
/// 
/// 
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
                    let data = match create_test(event.value().to_string()).await {
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

/// Echo the user input on the server.
// #[server(EchoServer)]
// async fn echo_server(input: String) -> Result<String, ServerFnError> {
//     Ok(input)
// }

#[server]
pub async fn create_test(input: String) -> Result<String, ServerFnError> {
    let db = s_helpers::db::get_db_pool().await;
    Ok(format!("Created test with input: {}", input))
}
