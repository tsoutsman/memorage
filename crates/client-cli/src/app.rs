use std::{net::IpAddr, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate the data and configuration files
    Setup {
        /// Save the generated config file to the specified path.
        #[clap(long)]
        config_output: Option<PathBuf>,
        /// Save the generated data file to the specified path.
        #[clap(long)]
        data_output: Option<PathBuf>,
    },
    Login {
        /// Save the generated config file to the specified path.
        #[clap(long)]
        config_output: Option<PathBuf>,
        /// Save the generated data file to the specified path.
        #[clap(long)]
        data_output: Option<PathBuf>,
    },
    /// Pair to a peer
    ///
    /// One peer runs the command without a code, generating a new code. The
    /// other peer runs the command with this newly generated code. Peers must
    /// use the same coordination server.
    Pair {
        code: Option<memorage_cs::PairingCode>,
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    Sync {
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
        #[clap(long)]
        no_send: bool,
        #[clap(long)]
        no_receive: bool,
    },
    Daemon {
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
    /// Retrieve stored files
    Retrieve {
        /// Place retrieved files in the specified directory
        #[clap(short, long)]
        output: Option<PathBuf>,
        /// Use the specified configuration file
        #[clap(short, long)]
        config: Option<PathBuf>,
        /// Use the specified data file
        #[clap(short, long)]
        data: Option<PathBuf>,
        /// Use the specified coordination server
        ///
        /// The address can be IPv4 or IPv6.
        #[clap(short, long)]
        server: Option<IpAddr>,
    },
}
