use comfy_table::{presets::UTF8_FULL_CONDENSED, ContentArrangement, Table};
use owo_colors::OwoColorize;
use serde::Serialize;

pub fn output<T: Serialize>(json_mode: bool, data: &T, render_table: impl FnOnce(&T)) {
    if json_mode {
        match serde_json::to_string_pretty(data) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("{} Failed to serialize JSON: {e}", "error:".red().bold()),
        }
    } else {
        render_table(data);
    }
}

pub fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers);
    table
}

pub fn section_header(title: &str) {
    println!("\n{}", title.cyan().bold());
    println!("{}", "─".repeat(title.len()).cyan());
}

pub fn success(msg: &str) {
    println!("{} {msg}", "✓".green().bold());
}

pub fn warn(msg: &str) {
    eprintln!("{} {msg}", "warning:".yellow().bold());
}

pub fn error(msg: &str) {
    eprintln!("{} {msg}", "error:".red().bold());
}
