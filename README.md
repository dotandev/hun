# Hun: The History Unification Node 🏹

**Hun** is a supercharged shell history tool written in Rust. It replaces your standard shell history with a SQLite-backed database, offering better persistence, metadata tracking (exit codes, execution time, directory), and a powerful TUI for fuzzy searching.

![Hun TUI](https://via.placeholder.com/800x400?text=Hun+TUI+Screenshot)

## Features

*   **SQLite Backend**: Your history is stored in a robust SQL database, not a fragile text file.
*   **Rich Metadata**: Tracks **Exit Code** (✅/❌), **Timestamp**, **Current Working Directory**, and **Session ID**.
*   **Interactive TUI**: Built with `ratatui`, offering instant fuzzy search.
*   **Shell Integration**: Seamless hooks for Zsh and Bash.
*   **Stats**: View your most frequently used commands with `hun stats`.

## Installation

### From Crates.io

```bash
cargo install hun
```

### From Source

```bash
git clone https://github.com/dotandev/hun.git
cd hun
cargo install --path .
```

## Setup

To start recording commands, you need to hook `hun` into your shell.

### Zsh

Add the following to your `~/.zshrc`:

```zsh
# Hun Integration
function hun_add_history() {
    local EXIT_CODE=$?
    local CMD=$(fc -ln -1)
    # Run in background to avoid latency
    hun add --cmd "$CMD" --cwd "$PWD" --exit-code "$EXIT_CODE" &!
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd hun_add_history

# Bind Ctrl+R to Hun Search
function hun_search() {
    local SELECTED_CMD=$(hun search --query "$BUFFER")
    if [ -n "$SELECTED_CMD" ]; then
        BUFFER="$SELECTED_CMD"
        CURSOR=$#BUFFER
    fi
    zle redisplay
}
zle -N hun_search
bindkey "^R" hun_search
```

### Bash

Add the following to your `~/.bashrc`:

```bash
# Hun Integration
function hun_add_history() {
    local EXIT_CODE=$?
    local CMD=$(history 1 | sed 's/^[ ]*[0-9]\+[ ]*//')
    hun add --cmd "$CMD" --cwd "$PWD" --exit-code "$EXIT_CODE" &
}

PROMPT_COMMAND="hun_add_history; $PROMPT_COMMAND"

# Bind Ctrl+R to Hun Search
bind -x '"\C-r": "READLINE_LINE=$(hun search --query \"$READLINE_LINE\") && READLINE_POINT=${#READLINE_LINE}"'
```

## Usage

### Search History
Press `Ctrl+R` (if configured) or run:
```bash
hun search
```
*   **Type** to filter.
*   **Up/Down** to navigate.
*   **Enter** to select.
*   **Esc** to cancel.

### View Stats
See your top 10 most used commands:
```bash
hun stats
```

### Manual Add
You generally don't need this, but you can add entries manually:
```bash
hun add --cmd "echo hello" --cwd "/tmp" --exit-code 0
```

## License

MIT
