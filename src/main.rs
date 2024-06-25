use fssh::{retrive_ssh_configs, App, Terminal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let configs = retrive_ssh_configs()?;
    let mut app = App::new(configs);

    let mut terminal = Terminal::new(app.data.len() as u16 + 5)?;

    app.run(&mut terminal)?;

    // drop is needed to cleanup the terminal
    drop(terminal);

    app.connect()?;

    Result::Ok(())
}
