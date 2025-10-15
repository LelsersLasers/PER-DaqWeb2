use crate::routes;
use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Navbar() -> Element {
    rsx! {
        div {
            Link {
                to: routes::Route::Home {},
                "Home"
            }
            Link {
                to: routes::Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<routes::Route> {}
    }
}
