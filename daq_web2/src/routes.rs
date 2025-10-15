use dioxus::prelude::*;

use crate::ui::upload::Upload;
use crate::ui::home::Home;
use crate::ui::navbar::Navbar;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/upload")]
    Upload {},
}
