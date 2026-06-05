use std::io::{self, Write};

use daken_rs::{KeyResult, RomajiInput};

const DEFAULT_TARGETS: &[&str] = &["タイピング", "しんぶん", "きょうはいいてんき", "マッチ"];

fn main() -> io::Result<()> {
    println!("daken-rs console example");
    println!("Type romaji for the kana target. Submit one line at a time.");
    println!();

    let target = choose_target()?;
    let mut input = RomajiInput::new(target);
    let mut misses = 0usize;

    loop {
        render(&input, misses);

        let Some(line) = prompt("input> ")? else {
            println!("input ended before completion.");
            return Ok(());
        };
        for key in line.chars() {
            match input.input(key) {
                KeyResult::Accepted => {}
                KeyResult::Completed => {
                    render(&input, misses);
                    println!("completed!");
                    return Ok(());
                }
                KeyResult::Rejected => {
                    misses += 1;
                    println!("miss: `{key}`");
                }
            }
        }
    }
}

fn choose_target() -> io::Result<String> {
    println!("Targets:");
    for (index, target) in DEFAULT_TARGETS.iter().enumerate() {
        println!("  {}. {}", index + 1, target);
    }
    println!("  custom. Enter your own kana text");
    println!();

    let Some(selected) = prompt("target number or text> ")? else {
        return Ok(DEFAULT_TARGETS[0].to_string());
    };
    let trimmed = selected.trim();

    if let Ok(number) = trimmed.parse::<usize>() {
        if let Some(target) = DEFAULT_TARGETS.get(number.saturating_sub(1)) {
            return Ok((*target).to_string());
        }
    }

    if trimmed.is_empty() {
        Ok(DEFAULT_TARGETS[0].to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn render(input: &RomajiInput, misses: usize) {
    println!();
    println!("target : {}", input.target());
    println!("typed  : {}", input.typed());
    println!(
        "next   : {}",
        input.next_keys().into_iter().collect::<String>()
    );
    println!("misses : {misses}");
}

fn prompt(label: &str) -> io::Result<Option<String>> {
    print!("{label}");
    io::stdout().flush()?;

    let mut line = String::new();
    let bytes_read = io::stdin().read_line(&mut line)?;
    if bytes_read == 0 {
        Ok(None)
    } else {
        Ok(Some(line.trim_end().to_string()))
    }
}
