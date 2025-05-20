use console::{Style, StyledObject};
use std::fmt::Display;

// Text styles for console output

pub fn upd_sty<S: Display>(text: S) -> String {
    format![
        "{}",
        Style::new()
            .for_stderr()
            .cyan()
            .bright()
            .apply_to(text.to_string())
    ]
}

pub fn info_sty_str<S: Display>(text: S) -> StyledObject<String> {
    Style::new()
        .for_stderr()
        .white()
        .italic()
        .apply_to(text.to_string())
}

pub fn info_sty_num<S: Display>(n: S) -> StyledObject<String> {
    Style::new()
        .for_stderr()
        .yellow()
        .bright()
        .apply_to(n.to_string())
}
