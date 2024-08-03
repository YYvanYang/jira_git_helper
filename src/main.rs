use clap::{Command, Arg, ArgAction};
use jira_git_helper::{AppError, App, app_config};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let matches = Command::new("JIRA Git Helper")
        .version("1.0")
        .author("Your Name")
        .about("Automates JIRA-related Git commit tasks")
        // .arg(Arg::new("help")
        //     .short('h')
        //     .long("help")
        //     .help("Prints help information")
        //     .action(ArgAction::SetTrue))
        .arg(Arg::new("windows_help")
            .help("Prints help information (Windows style)")
            .short('?')
            .long("/?")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Configure JIRA Git Helper settings")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("reset")
            .short('r')
            .long("reset")
            .help("Reset all configurations")
            .action(ArgAction::SetTrue))
        .get_matches();

    if matches.get_flag("windows_help") {
        return handle_help_command();
    }

    if matches.get_flag("config") {
        return handle_config_command().await;
    }

    if matches.get_flag("reset") {
        return handle_reset_command().await;
    }

    match App::new().await {
        Ok(mut app) => app.run().await,
        Err(AppError::ConfigMissing) => {
            println!("Configuration is missing or incomplete. Let's set it up!");
            handle_config_command().await
        }
        Err(e) => Err(e),
    }
}

fn handle_help_command() -> Result<(), AppError> {
    println!("JIRA Git Helper");
    println!("Usage: jira_git_helper [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help, /?    Show this help message");
    println!("  -c, --config      Configure JIRA Git Helper settings");
    println!("  -r, --reset       Reset all configurations");
    println!();
    println!("Normal Usage:");
    println!("  1. Ensure you're in a Git repository.");
    println!("  2. Your branch name should include a JIRA issue ID (e.g., 'feature/PROJ-123-add-login').");
    println!("  3. Run 'jira_git_helper' without any arguments in your repository.");
    println!("  4. The tool will extract the JIRA ID, fetch the issue title, and guide you through the commit process.");
    println!();
    println!("For more detailed information, please refer to the README.md file.");

    Ok(())
}

async fn handle_config_command() -> Result<(), AppError> {
    println!("Starting JIRA Git Helper configuration...");
    app_config::create_interactive_config(None).await?;
    println!("Configuration updated successfully!");
    println!("You can now run the program again to use JIRA Git Helper.");
    Ok(())
}

async fn handle_reset_command() -> Result<(), AppError> {
    app_config::reset_config()?;
    println!("All configurations have been reset.");
    Ok(())
}