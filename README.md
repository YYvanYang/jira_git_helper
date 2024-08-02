# JIRA Git Helper

![Rust Version][rust-image]
![License][license-image]

JIRA Git Helper 是一个 Rust 编写的命令行工具，用于简化 Git 提交过程中与 JIRA 相关的任务。它可以从 Git 分支名称中提取 JIRA ID，获取相关的 JIRA issue 标题，并生成格式化的提交信息。

## 特性

- 自动从 Git 分支名称中提取 JIRA ID
- 与 JIRA API 集成，获取实时 issue 信息
- 安全存储 JIRA 凭证（密码经过加密）
- 支持通过配置文件和环境变量进行灵活配置
- 异步操作，提高性能
- 交互式提交信息编辑

## 安装

### 从 crates.io 安装

```bash
cargo install jira_git_helper
```

### 从源代码安装

```bash
git clone https://github.com/YYvanYang/jira_git_helper.git
cd jira_git_helper
cargo install --path .
```

确保 `~/.cargo/bin` 在您的 PATH 中。

## 配置

### 交互式配置

首次运行时，工具会提示输入必要的配置信息：

```bash
jira_git_helper
```

### 配置文件

配置存储在 `~/.jira_git_helper.toml` 文件中：

```toml
jira_url = "https://your-jira-instance.atlassian.net"
username = "your_username"
encrypted_password = "encrypted_password_string"
jira_id_prefix = "PROJ"
```

### 环境变量

也可以使用环境变量进行配置：

```bash
export JIRA_GIT_JIRA_URL=https://your-jira-instance.atlassian.net
export JIRA_GIT_USERNAME=your_username
export JIRA_GIT_PASSWORD=your_password
export JIRA_GIT_JIRA_ID_PREFIX=PROJ
```

### 重置配置

```bash
jira_git_helper reset
```

## 使用方法

1. 确保您的 Git 分支名称包含 JIRA ID (例如 `feature/PROJ-1234-add-login`)。

2. 在 Git 仓库目录中运行：
   ```
   jira_git_helper
   ```

3. 工具会自动提取 JIRA ID，获取 issue 标题，并提示您输入额外的提交信息。

4. 确认生成的提交信息后，工具会执行 `git commit` 命令。

## 本地开发

### 环境准备

1. 确保已安装 Rust 和 Cargo。
2. 克隆仓库：
   ```bash
   git clone https://github.com/YYvanYang/jira_git_helper.git
   cd jira_git_helper
   ```

### 构建和运行

```bash
# 安装依赖并构建
cargo build

# 运行开发版本
cargo run

# 运行并传递参数
cargo run -- reset

# 运行测试
cargo test

# 代码检查和格式化
cargo clippy
cargo fmt

# 构建发布版本
cargo build --release
```

### 开发配置

创建 `.env` 文件用于开发环境：

```
JIRA_GIT_JIRA_URL=https://your-dev-jira-instance.atlassian.net
JIRA_GIT_USERNAME=your_dev_username
JIRA_GIT_PASSWORD=your_dev_password
JIRA_GIT_JIRA_ID_PREFIX=DEV
```

### 调试

使用 VS Code 的 `.vscode/launch.json`：

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug executable 'jira_git_helper'",
      "cargo": {
        "args": ["build", "--bin=jira_git_helper", "--package=jira_git_helper"],
        "filter": {
          "name": "jira_git_helper",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

启用详细日志：

```bash
RUST_LOG=debug cargo run
```

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解更多信息。

## 许可证

本项目采用 MIT 许可证。详情请见 [LICENSE](LICENSE) 文件。

[rust-image]: https://img.shields.io/badge/rust-1.70%2B-blue.svg
[license-image]: https://img.shields.io/badge/License-MIT-blue.svg
