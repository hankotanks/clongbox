#![warn(clippy::all, rust_2018_idioms)]
#![feature(if_let_guard)]
#![feature(impl_trait_in_assoc_type)]

mod app;
pub use app::App;
pub use app::fonts::FONT_ID;

mod sub;
pub use sub::widgets;

mod state;
pub use state::State;
pub use state::focus::{Focus, FocusTarget, FocusBuffer};

mod types;
pub use types::language;
pub use types::group::{Group, GroupKey, GroupName};
pub use types::phoneme::{Phoneme, PhonemeKey};
pub use types::sc;
pub use types::selection::Selection;

mod panes;
pub use panes::Pane;
pub use panes::panes;

mod tools;
pub use tools::Tool;
pub use tools::tools;