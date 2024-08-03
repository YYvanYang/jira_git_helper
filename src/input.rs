use std::io::{self, Write};
use rpassword::read_password;

pub fn prompt_for_input(prompt: &str, default: Option<&str>) -> io::Result<String> {
    let default_display = default.map_or_else(String::new, |d| format!(" [{}]", d));
    print!("{}{}: ", prompt, default_display);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() && default.is_some() {
        Ok(default.unwrap().to_string())
    } else {
        Ok(input.to_string())
    }
}

pub fn prompt_for_password(prompt: &str) -> io::Result<String> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    read_password()
}

pub fn prompt_for_commit_message() -> String {
    prompt_for_input("Enter additional commit message", None).unwrap_or_default()
}

pub fn confirm_commit(commit_message: &str) -> bool {
    println!("Git commit command: git commit -m \"{}\"", commit_message);
    let input = prompt_for_input("Do you want to proceed? (y/n)", Some("y")).unwrap_or_default();
    matches!(input.to_lowercase().as_str(), "y" | "yes" | "")
}