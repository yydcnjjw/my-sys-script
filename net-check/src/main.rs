use clap::Clap;
use futures::future::{BoxFuture, FutureExt};
use run_script::ScriptOptions;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::TcpSocket;
use tokio::time::{self, timeout, Duration};

#[derive(Clap)]
#[clap(version("0.1.0"), author("yydcnjjw <yydcnjjw@gmail.com>"))]
pub struct Opts {
    /// conf path
    #[clap(required(true), index(1))]
    pub conf_path: String,
    /// interval
    #[clap(short, long, default_value("60"))]
    pub interval: u64,
}

fn policy_timeout_default() -> u64 {
    20
}

#[derive(Serialize, Deserialize, Debug)]
struct Policy {
    addr: String,
    #[serde(default)]
    success_cmd: String,
    #[serde(default)]
    failure_cmd: String,
    #[serde(default = "policy_timeout_default")]
    timeout: u64,
    #[serde(default)]
    sub_policy: Vec<Policy>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("policy parse error: {0}")]
    PolicyParse(String),
}

type Result<T> = std::result::Result<T, Error>;
async fn exec_cmd(cmd: &String) -> Result<()> {
    if cmd.is_empty() {
        return Ok(());
    }

    let options = ScriptOptions::new();
    let args = vec![];

    let (exit_code, _, err) = run_script::run(cmd, &args, &options).unwrap();

    if exit_code != 0 {
        log::warn!("{}", err);
    }
    
    Ok(())
}

fn exec_policy<'a>(policies: &'a Vec<Policy>) -> BoxFuture<'a, Result<()>> {
    async move {
        for policy in policies {
            let addr = policy
                .addr
                .parse::<SocketAddr>()
                .map_err(|e| Error::PolicyParse(e.to_string()))?;

            let socket = TcpSocket::new_v4().map_err(|e| Error::PolicyParse(e.to_string()))?;
            match timeout(Duration::from_secs(policy.timeout), socket.connect(addr)).await {
                Ok(v) => match v {
                    Ok(_s) => {
                        exec_cmd(&policy.success_cmd).await?;
                        exec_policy(&policy.sub_policy).await?
                    }
                    Err(_) => exec_cmd(&policy.failure_cmd).await?,
                },
                Err(_) => exec_cmd(&policy.failure_cmd).await?,
            }
        }
        Ok(())
    }
    .boxed()
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    let conf_file = File::open(opts.conf_path).map_err(|e| Error::PolicyParse(e.to_string()))?;

    let policies: Vec<Policy> = serde_yaml::from_reader(conf_file).unwrap();
    println!("{:?}", policies);

    let mut interval = time::interval(time::Duration::from_secs(opts.interval));

    loop {
        interval.tick().await;
        exec_policy(&policies).await?;
    }
}
