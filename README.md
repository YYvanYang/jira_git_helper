# JIRA Git Helper

A Rust-based command-line tool to automate the generation of Git commit messages with JIRA information. This tool extracts JIRA ID and title from the current branch name, prompts for additional commit message details, and ensures secure storage of JIRA credentials.

## Features

1. Automatically retrieves JIRA ID and title from the current branch name.
2. Prompts the user to input additional commit message details.
3. Executes the `git commit` command with a formatted message.
4. Ensures secure storage of JIRA credentials.

## Prerequisites

Ensure your current Git branch name includes the JIRA ID, e.g., `feature/JIRA-1234-add-login-feature`.

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/YYvanYang/jira_git_helper.git
    cd jira_git_helper
    ```

2. Build and install the tool:
    ```sh
    cargo build --release
    cargo install --path .
    ```

## Configuration

On the first run, the tool will prompt you to enter necessary configuration details including JIRA URL, username, password, and JIRA ID prefix.

Run the tool:
```sh
jira_git_helper
```

Example output:
```plaintext
Enter your JIRA URL: [ ]
Enter your domain username: [ ]
Enter your domain password:
Enter your JIRA ID prefix: [JIRA]
```

## Usage

Ensure your current Git branch includes the JIRA ID, e.g., `feature/JIRA-1234-add-login-feature`.

Run the tool:
```sh
jira_git_helper
```

Example output:
```plaintext
JIRA ID: JIRA-1234
JIRA Title: Add login feature
Enter additional commit message: Fixed login bug
Git commit command: git commit -m "[JIRA-1234] Add login feature Fixed login bug"
Do you want to proceed? (y/n) [y]:
```

This generates and executes the following Git commit command:
```sh
git commit -m "[JIRA-1234] Add login feature Fixed login bug"
```

## Reset Configuration

To reset the configuration, use the `reset` command:
```sh
jira_git_helper reset
```

Example output:
```plaintext
Configuration reset successfully.
```

## Testing

### Unit Tests

Run unit tests:
```sh
cargo test --test unit_tests
```

### Integration Tests

Run integration tests:
```sh
cargo test --test integration_test
```

## Summary

This command-line tool, built with Rust, automates the generation of Git commit messages with JIRA information. It simplifies the commit process and ensures secure storage of JIRA credentials, significantly improving development efficiency and reducing manual errors.

## License

This project is licensed under the MIT License.
