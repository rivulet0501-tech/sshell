# sshell MVP Acceptance Checklist

## Prerequisites

- A reachable SSH server with shell access
- Test programs installed remotely: `vim`, `top`, `less`, `htop`
- A Qt caller or shell wrapper that can inspect process exit codes

## Manual test 1: Single-session startup

Run:

```bash
cargo run -- root:secret@127.0.0.1
```

Expected:

- The TUI opens with one tab
- The remote shell accepts typed commands
- `Ctrl+A` by itself does not close the program

## Manual test 2: Full-screen programs

Run each command remotely:

```bash
vim
top
less /etc/hosts
htop
```

Expected for each:

- The screen updates without corruption
- Leaving the program returns to the shell prompt
- Resizing the terminal updates the remote layout

## Manual test 3: Multi-tab startup

Run:

```bash
cargo run -- root:secret@127.0.0.1 root:secret@example.com:2222
```

Expected:

- Two tabs are visible on startup
- `Ctrl+A`, `n` moves to the next tab
- `Ctrl+A`, `p` moves to the previous tab
- `Ctrl+A`, `x` closes only the active tab

## Manual test 4: Exit codes

Run:

```bash
cargo run -- root:wrong@127.0.0.1 ; echo $?
```

Expected: stderr contains `[ERROR] 4:` and the shell prints `4`
