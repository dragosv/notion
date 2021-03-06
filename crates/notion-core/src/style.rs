//! The view layer of Notion, with utilities for styling command-line output.

use std::env;
use std::fmt::{self, Display, Formatter};

use console::style;
use failure::Fail;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

/// Represents the context from which an error is being reported.
pub enum ErrorContext {
    /// An error reported from the `notion` executable.
    Notion,

    /// An error reported from a shim.
    Shim,
}

/// Displays an error to stderr.
pub fn display_error<E: Display>(cx: ErrorContext, err: &E) {
    display_error_prefix(cx);
    eprintln!("{}", err);
}

/// Displays an error to stderr with a styled prefix.
pub fn display_error_prefix(cx: ErrorContext) {
    match cx {
        ErrorContext::Notion => {
            // Since the command here was `notion`, it would be redundant to say that this was
            // a Notion error, so we are less explicit in the heading.
            eprint!("{} ", style("error:").red().bold());
        }
        ErrorContext::Shim => {
            // Since a Notion error is rare case for a shim, it can be surprising to a user.
            // To make it extra clear that this was a failure that happened in Notion when
            // attempting to delegate to a shim, we are more explicit about the fact that it's
            // a Notion error.
            eprint!("{} ", style("Notion error:").red().bold());
        }
    }
}

/// Displays a generic message for internal errors to stderr.
pub fn display_unknown_error<E: Fail>(cx: ErrorContext, err: &E) {
    display_error_prefix(cx);
    eprintln!("an internal error occurred");
    eprintln!();

    if env::var("NOTION_DEV").is_ok() {
        eprintln!("{} {:?}", style("details:").yellow().bold(), err);
        eprintln!();

        let backtrace = err.backtrace();

        // For now, we require RUST_BACKTRACE for this to work.
        // See: https://github.com/notion-cli/notion/issues/75

        if backtrace.is_some() && env::var("RUST_BACKTRACE").is_ok() {
            eprintln!("{:?}", backtrace.unwrap());
        } else {
            eprintln!("Run with NOTION_DEV=1 and RUST_BACKTRACE=1 for a backtrace.");
        }
    } else {
        eprintln!("Notion is still a pre-alpha project, so we expect to run into some bugs,");
        eprintln!("but we'd love to hear about them so we can fix them!");
        eprintln!();
        eprintln!(
            "Please feel free to reach out to us at {} on Twitter or file an issue at:",
            style("@notionjs").cyan().bold()
        );
        eprintln!();
        eprintln!(
            "    {}",
            style("https://github.com/notion-cli/notion/issues").bold()
        );
        eprintln!();
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum Action {
    Fetching,
}

impl Action {
    // this is the maximum width of the displayed Action strings, used for formatting
    const MAX_WIDTH: usize = 10;
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            &Action::Fetching => "Fetching",
        };
        f.write_str(s)
    }
}

/// Constructs a command-line progress bar with the specified Action enum
/// (e.g., `Action::Installing`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(action: Action, details: &str, len: u64) -> ProgressBar {
    let display_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
    let msg_width = Action::MAX_WIDTH + 1 + details.len();

    //   Installing v1.23.4  [====================>                   ]  50%
    // |----------| |-----|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let available_width = display_width - 2 - msg_width - 2 - 2 - 1 - 3 - 1;
    let bar_width = ::std::cmp::min(available_width, 40);

    let bar = ProgressBar::new(len);

    bar.set_message(&format!(
        "{: >width$} {}",
        style(action.to_string()).green().bold(),
        details,
        width = Action::MAX_WIDTH
    ));
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%",
                bar_width
            ))
            .progress_chars("=> "),
    );

    bar
}

/// Constructs a command-line progress spinner with the specified "message"
/// string. The spinner is ticked by default every 20ms.
pub fn progress_spinner(message: &str) -> ProgressBar {
    // ⠋ Fetching public registry: https://nodejs.org/dist/index.json
    let spinner = ProgressBar::new_spinner();

    spinner.set_message(message);
    spinner.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}"));
    spinner.enable_steady_tick(20); // tick the spinner every 20ms

    spinner
}
