use dioxus::prelude::*;


#[component]
#[allow(non_snake_case)]
pub fn Navbar() -> Element {
    rsx! {
        div {
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