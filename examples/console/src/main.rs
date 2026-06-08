use std::io::{self, Write};

use daken_rs::{KeyResult, TypingSession};

const DEFAULT_TARGETS: &[&str] = &["タイピング", "しんぶん", "きょうはいいてんき", "マッチ"];

fn main() -> io::Result<()> {
    println!("daken-rs console example");
    println!("Type romaji for the kana target. Submit one line at a time.");
    println!();

    let target = choose_target()?;
    let mut session = TypingSession::new(target);

    loop {
        render(&session);

        let Some(line) = prompt("input> ")? else {
            println!("input ended before completion.");
            return Ok(());
        };
        for key in line.chars() {
            match session.input(key) {
                KeyResult::Accepted => {}
                KeyResult::Completed => {
                    render(&session);
                    println!("completed!");
                    return Ok(());
                }
                KeyResult::Rejected => {
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

    if let Ok(number) = trimmed.parse::<usize>()
        && let Some(target) = DEFAULT_TARGETS.get(number.saturating_sub(1))
    {
        return Ok((*target).to_string());
    }

    if trimmed.is_empty() {
        Ok(DEFAULT_TARGETS[0].to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn render(session: &TypingSession) {
    let matcher = session.matcher();
    let progress = matcher.progress();

    println!();
    println!("target : {}", matcher.target());
    println!("typed  : {}", matcher.typed());
    println!(
        "progress: {}/{} target chars, {} typed keys",
        progress.confirmed_target_chars, progress.total_target_chars, progress.typed_keys
    );
    println!(
        "next   : {}",
        matcher.next_keys().into_iter().collect::<String>()
    );
    println!(
        "remain : {}",
        matcher
            .remaining_romaji_candidates()
            .into_iter()
            .take(5)
            .collect::<Vec<_>>()
            .join(" / ")
    );
    println!("misses : {}", session.misses());
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
