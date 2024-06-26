use std::error::Error;

use fssh::{retrive_ssh_configs, App};

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new(retrive_ssh_configs()?);
    app.run()?;

    Ok(())
}
