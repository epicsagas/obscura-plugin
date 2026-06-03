use clap::{Parser, Subcommand};

use obscura_plugin::{install, wizard};

#[derive(Parser)]
#[command(
    name = "obscura-plugin",
    about = "Installer for Obscura headless browser skills"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install skills to AI coding tools
    Install {
        /// Tool: cursor, opencode, cline, all
        tool: Option<String>,
    },

    /// Uninstall skills from AI coding tools
    Uninstall {
        /// Tool: cursor, opencode, cline, all
        tool: String,
    },

    /// List supported tools
    List,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { tool } => match tool {
            None => {
                let selected = wizard::interactive_select();
                if selected.is_empty() {
                    println!("\n  Cancelled.\n");
                } else {
                    for t in &selected {
                        install::install_tool(t);
                    }
                }
            }
            Some(t) if t == "all" => {
                for (id, _) in install::ALL_TOOLS {
                    install::install_tool(id);
                }
            }
            Some(t) => {
                for t in t.split(',') {
                    install::install_tool(t.trim());
                }
            }
        },
        Commands::Uninstall { tool } => {
            if tool == "all" {
                for (id, _) in install::ALL_TOOLS {
                    install::uninstall_tool(id);
                }
            } else {
                for t in tool.split(',') {
                    install::uninstall_tool(t.trim());
                }
            }
        }
        Commands::List => install::list_tools(),
    }
}
