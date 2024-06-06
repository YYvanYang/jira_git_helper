# JIRA Git Helper

A CLI tool to automate Git commit messages with JIRA integration.

## Installation

Install the tool using Cargo:

```sh
cargo install jira_git_helper
```

## Usage

Assuming your current Git branch name is `feature/JIRA-1234-add-login-feature`:

1. **First Run**: The tool will prompt you to enter your JIRA URL, username, and password. These will be saved in a configuration file for future use.

```sh
$ jira_git_helper
Enter your JIRA URL: https://your-jira-domain.com
Enter your domain username: your_domain_username
Enter your domain password:
```

2. **Subsequent Runs**: The tool will automatically use the saved JIRA URL, username, and password.

```sh
$ jira_git_helper
JIRA ID: JIRA-1234
JIRA Title: Add login feature
Enter additional commit message: Fixed login bug
```

The tool will then generate and execute the following Git commit command:

```sh
git commit -m "[JIRA-1234] Add login feature Fixed login bug"
```

## License

This project is licensed under the MIT License.
