use dioxus::prelude::*;

use crate::ui::home::Home;
use crate::ui::navbar::Navbar;
use crate::ui::upload::Upload;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/upload")]
    Upload {},
}
