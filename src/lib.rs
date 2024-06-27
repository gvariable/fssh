#![deny(missing_docs)]
//! A CLI tool for quickly connecting to SSH servers with an intuitive TUI interface.
//!
//! # Features
//! - Intuitive TUI interface for selecting and searching from a large list of SSH servers.
//! - Automatically memorizes and encrypts passwords, requiring password entry only once.
//!
//! # Usage
//! ```shell
//! $ fssh
//! ```
//!
//! ## How it works
//! 1. `fssh` parses your `~/.ssh/config` file and lists all the hosts.
//! 2. Users can search for and select the host they want to connect to.
//! 3. `fssh` spawns a new TTY and runs the SSH client to connect to the chosen host.
//! 4. If the host requires a password, `fssh` will memorize and encrypt it locally. The next time the user connects to the same host, they won't need to enter the password again.
//! 5. If the host doesn't require a password, `fssh` will connect directly.
mod app;
mod db;
mod encrypt;
mod input;
mod pty;
mod select_box;
mod sshconfig;
mod terminal;

pub use app::App;
pub use db::Db;
pub use encrypt::EncryptionManager;
pub use pty::{CommandBuilder, PseudoTerminal};
pub use select_box::SelectBox;
pub use sshconfig::*;
pub use terminal::Terminal;
