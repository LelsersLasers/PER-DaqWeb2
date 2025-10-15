use dioxus::prelude::*;

use crate::ui;
use crate::ui::blog::Blog;
use crate::ui::home::Home;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(ui::navbar::Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}
