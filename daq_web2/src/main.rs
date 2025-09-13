mod assets;
mod config;

use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
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

    println!("Listening on http://{ip}:{port}");

    let address = std::net::SocketAddr::new(ip, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::default(), App)
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
#[component]
#[allow(non_snake_case)]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
#[allow(non_snake_case)]
fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[server(EchoServer)]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
