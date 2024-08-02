use std::io::{self, Write};

pub fn prompt_for_input(prompt: &str, default: Option<&str>) -> String {
    print!("{} [{}]: ", prompt, default.unwrap_or(""));
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_string();
    if input.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        input
    }
}

pub fn prompt_for_commit_message() -> String {
    prompt_for_input("Enter additional commit message", None)
}

pub fn confirm_commit(commit_message: &str) -> bool {
    println!("Git commit command: git commit -m \"{}\"", commit_message);
    let input = prompt_for_input("Do you want to proceed? (y/n)", Some("y"));
    matches!(input.to_lowercase().as_str(), "y" | "yes" | "")
}
