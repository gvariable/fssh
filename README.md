<div align="center">
    <img src="logo.webp" alt="FSSH Logo" width="150" height="150" /> <!-- Replace with the actual path to the logo -->
    <br/>
    <b>Connect quickly to your SSH servers 🚀</b>
    <br/>
    <br/>
    <a href="https://github.com/gvariable/fssh/actions/workflows/release.yml">
        <img src="https://github.com/gvariable/fssh/actions/workflows/release.yml/badge.svg" alt="Release Status" />
    </a>
    </a>
    <img src="https://img.shields.io/crates/l/fssh.svg" alt="License">
    <br/>
    <br/>
    <div>
        FSSH is a TUI tool that allows you to quickly connect to your SSH servers by navigating through your SSH config.
    </div>
    <br/>
</div>

# FSSH

A CLI tool for quickly connecting to SSH servers with an intuitive TUI interface.

![Demo](./demo.gif)

## Features

- Intuitive TUI interface for selecting and searching from a large list of SSH servers.
- Automatically memorizes and encrypts passwords, requiring password entry only once.

## Usage

```shell
$ fssh
```



## How It Works

1. `fssh` parses your `~/.ssh/config` file and lists all the hosts.
2. Users can search for and select the host they want to connect to.
3. `fssh` spawns a new TTY and runs the SSH client to connect to the chosen host.
4. If the host requires a password, `fssh` will memorize and encrypt it locally. The next time the user connects to the same host, they won't need to enter the password again.
5. If the host doesn't require a password, `fssh` will connect directly.


Feel free to contribute to the project by opening issues or submitting pull requests. We appreciate your feedback and help to improve `fssh`.
