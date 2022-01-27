use std::collections::HashMap;
use std::thread;

use clap::Parser;
use tokio::time::{Duration, sleep};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "cloudflare")]
    platform: String,

    #[clap(long, default_value = "")]
    zone: String,
    #[clap(long, default_value = "")]
    auth_email: String,
    #[clap(long, default_value = "")]
    auth_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    println!("{:?}", args);

    let mut ip = String::new();
    let mut count = 0;
    loop {
        //查询IP
        let response = reqwest::get("https://httpbin.org/ip")
            .await?
            .json::<HashMap<String, String>>()
            .await?;

        if response["origin"] != ip {
            ip = response["origin"].clone();
            //更新远程IP
            println!("{}", ip);
        };

        println!("循环次数:{}", count);
        //休眠
        sleep(Duration::from_secs(10));
    }
}
