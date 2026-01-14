use inquire::Text;
use inquire::formatter::StringFormatter;
use inquire::ui::{RenderConfig, Styled, StyleSheet, Color};
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

pub fn get_input_text(
    question: &str,
    default: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let render_config = RenderConfig::default_colored()
        .with_prompt_prefix(Styled::new("?").with_fg(Color::LightCyan))
        .with_answered_prompt_prefix(Styled::new("√").with_fg(Color::LightGreen))
        .with_text_input(
            StyleSheet::default().with_fg(Color::Grey),
        );

    let formatter: StringFormatter = &|i: &str| {
        if i.is_empty() {
            format!("\x1b[90m{}\x1b[0m {}", "...", default)
        } else {
            format!("\x1b[90m{}\x1b[0m {}", "...", i)
        }
    };


    let spaces = " ".repeat(default.len());

    let input = Text::new(question)
        .with_placeholder(&format!("{} {}", "»", default).to_string())
        .with_render_config(render_config)
        .with_formatter(formatter)
        .prompt()?;

    let input = if input.is_empty() {
        default.to_string()
    } else {
        input
    };

    Ok(input)
}

pub fn show_loader(message: &str, running: Arc<AtomicBool>) {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    
    while running.load(Ordering::Relaxed) {
        pb.tick();
        thread::sleep(Duration::from_millis(100));
    }
    
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("\x1b[32m√\x1b[0m {msg}")
            .unwrap(),
    );
    pb.finish_with_message(format!("{} Done.", message));
}

