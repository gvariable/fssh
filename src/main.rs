use fssh::{retrive_ssh_configs, App, Db, SshConfigItem, Terminal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let configs = retrive_ssh_configs()?;
    let mut app = App::new(configs);

    let mut terminal = Terminal::new(app.data.len() as u16 + 5)?;

    let selected = app.select(&mut terminal)?;

    if let Some(item) = selected {
        let path = dirs::config_dir().unwrap().join("fssh").join("db");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("db path: {:?}", path);
        let mut db: Db<SshConfigItem, String> = Db::open(path)?;

        if let Some(passwd) = db.get(&item) {
            app.connect(&item, Some(passwd.clone()))?;
        } else {
            let passwd = app.connect(&item, None)?;
            println!("get passwd: {:?}", passwd);
            if let Some(passwd) = passwd {
                db.insert(item, passwd);
                db.flush()?;
            }
        }
    }

    // drop is needed to cleanup the terminal
    drop(terminal);

    Result::Ok(())
}
