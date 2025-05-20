use console::{Style, StyledObject};
use std::fmt::Display;

mod cli;
pub use cli::Cli;

pub fn info_sty_str<S: Display>(text: S) -> StyledObject<String> {
    Style::new()
        .for_stderr()
        .white()
        .italic()
        .apply_to(text.to_string())
}

pub fn head_sty<S: Display>(text: S) -> StyledObject<String> {
    Style::new()
        .for_stderr()
        .cyan()
        .bold()
        .apply_to(text.to_string())
}
