use anyhow::Result;
use clap::Clap;
use std::{error::Error, path::PathBuf};

use crate::container::builder::ContainerBuilder;

#[derive(Clap, Debug)]
pub struct Exec {
    /// Unix socket (file) path , which will receive file descriptor of the writing end of the pseudoterminal
    #[clap(long)]
    pub console_socket: Option<PathBuf>,
    #[clap(short, long)]
    pub tty: bool,
    #[clap(long)]
    /// Current working directory of the container
    pub cwd: Option<PathBuf>,
    #[clap(long)]
    /// The file to which the pid of the container process should be written to
    pub pid_file: Option<PathBuf>,
    /// Environment variables that should be set in the container
    #[clap(short, long, parse(try_from_str = parse_key_val), number_of_values = 1)]
    pub env: Vec<(String, String)>,
    /// Prevent the process from gaining additional privileges
    #[clap(long)]
    pub no_new_privs: bool,
    /// Path to process.json
    #[clap(short, long)]
    pub process: Option<PathBuf>,
    /// Detach from the container process
    #[clap(short, long)]
    pub detach: bool,
    /// Identifier of the container
    pub container_id: String,
    /// Command that should be executed in the container
    #[clap(required = false)]
    pub command: Vec<String>,
}

impl Exec {
    pub fn exec(&self, root_path: PathBuf) -> Result<()> {
        let mut builder =
            ContainerBuilder::new(self.container_id.clone()).with_root_path(root_path);

        if let Some(console_socket) = &self.console_socket {
            builder = builder.with_console_socket(console_socket);
        }

        if let Some(pid_file) = &self.pid_file {
            builder = builder.with_pid_file(pid_file);
        }

        let mut builder = builder.as_tenant();

        if let Some(cwd) = &self.cwd {
            builder = builder.with_cwd(cwd);
        }

        if !self.env.is_empty() {
            let env = self.env.clone().into_iter().collect();
            builder = builder.with_env(env)
        }

        builder = builder.with_no_new_privs(self.no_new_privs);

        if let Some(process) = &self.process {
            builder = builder.with_process(process);
        }

        builder.with_container_args(self.command.clone()).build()?;

        Ok(())
    }
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}
