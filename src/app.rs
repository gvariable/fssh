use crate::pty::Size;
use crate::{
    select_box::SelectBox, sshconfig::SshConfigItem, terminal::Terminal, CommandBuilder, Db,
    PseudoTerminal,
};
use std::error::Error;

pub struct App {
    select_box: SelectBox,
}

impl App {
    pub fn new(data: Vec<SshConfigItem>) -> Self {
        App {
            select_box: SelectBox::new(data),
        }
    }

    fn select(&mut self) -> Result<Option<SshConfigItem>, Box<dyn Error>> {
        let mut terminal = Terminal::new()?;
        let selected = self.select_box.select(&mut terminal)?;
        Result::Ok(selected)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(item) = self.select()? {
            let path = dirs::config_dir().unwrap().join("fssh").join("db");
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            println!("db path: {:?}", path);
            let mut db: Db<SshConfigItem, String> = Db::open(path)?;

            let passwd = if let Some(passwd) = db.get(&item) {
                self.connect(&item, Some(passwd.clone()))?
            } else {
                self.connect(&item, None)?
            };

            println!("get passwd: {:?}", passwd);
            if let Some(passwd) = passwd {
                db.insert(item, passwd);
                db.flush()?;
            }
        }

        Result::Ok(())
    }

    fn connect(
        &self,
        hint: &SshConfigItem,
        passwd: Option<String>,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let mut terminal = Terminal::new()?;
        let mut cmd = CommandBuilder::new("ssh");
        cmd.arg(&hint.host);

        let size = Size::new(terminal.size()?.height, terminal.size()?.width);

        let rt = tokio::runtime::Runtime::new()?;
        let passwd = rt.block_on(async move {
            let mut pty = PseudoTerminal::new(size, cmd, passwd);
            pty.run(&mut terminal).await
        });

        Result::Ok(passwd)
    }
}
