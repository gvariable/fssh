use crate::pty::Size;
use crate::{
    select_box::SelectBox, sshconfig::SshConfigItem, terminal::Terminal, CommandBuilder, Db,
    EncryptionManager, PseudoTerminal,
};

const KEY_FILE: &str = "key";
const DB_FILE: &str = "db";
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// The entry to the CLI.
pub struct App {
    select_box: SelectBox,
}

impl App {
    /// Create a new [`App`] instance.
    pub fn new(data: Vec<SshConfigItem>) -> Self {
        App {
            select_box: SelectBox::new(data),
        }
    }

    /// Provide a TUI interface for selecting an SSH server.
    fn select(&mut self) -> anyhow::Result<Option<SshConfigItem>> {
        let mut terminal = Terminal::new(Some(self.select_box.len() as u16 + 5), false)?;
        let selected = self.select_box.select(&mut terminal)?;
        Result::Ok(selected)
    }

    /// Run the whole application.
    pub fn run(&mut self) -> anyhow::Result<()> {
        // select a host
        if let Some(item) = self.select()? {
            let db_path = dirs::config_dir().unwrap().join(CRATE_NAME).join(DB_FILE);
            let key_path = dirs::config_dir().unwrap().join(CRATE_NAME).join(KEY_FILE);

            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut db: Db<SshConfigItem, Vec<u8>> = Db::open(db_path)?;
            let manager = EncryptionManager::new(key_path)?;

            // check if the password is already stored
            let passwd = if let Some(passwd) = db.get(&item) {
                let passwd = manager.decrypt(&passwd)?;
                self.connect(&item, Some(String::from_utf8(passwd)?))?
            } else {
                self.connect(&item, None)?
            };

            // update the password
            if let Some(passwd) = passwd {
                db.insert(item, manager.encrypt(passwd.as_bytes())?);
                db.flush()?;
            }
        }

        Result::Ok(())
    }

    /// Spawn a new TTY and run the SSH client to connect to the chosen host.
    fn connect(
        &self,
        item: &SshConfigItem,
        passwd: Option<String>,
    ) -> anyhow::Result<Option<String>> {
        let mut terminal = Terminal::new(None, true)?;
        let mut cmd = CommandBuilder::new("ssh");
        cmd.arg(&item.host);

        let size = Size::new(terminal.size()?.height, terminal.size()?.width);

        let rt = tokio::runtime::Runtime::new()?;
        let passwd = rt.block_on(async move {
            let mut pty = PseudoTerminal::new(size, cmd, passwd)?;
            pty.run(&mut terminal).await
        })?;

        Result::Ok(passwd)
    }
}
