use dioxus::prelude::*;

mod assets;
mod backend;
mod config;
mod routes;
mod ui;

#[cfg(feature = "server")]
mod s_helpers;

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
        .serve_dioxus_application(ServeConfig::default(), component)
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
        Router::<routes::Route> {}
    }
}
