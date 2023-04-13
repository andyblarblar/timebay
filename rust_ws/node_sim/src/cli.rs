use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Node ids to use.
    #[arg(short, long, required = true)]
    pub node_ids: Vec<u16>,
    /// Hostname or IP address of the broker to connect to.
    pub broker_host: String,
}

pub const MENU_DIALOG: &str = r"
Commands:
1. Connect a node
2. Dissconnect a node
3. Trigger node
";
