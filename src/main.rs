use std::error::Error;

use chrono::Local;
use clap::Parser;
use serde_json::{json, Value};
use tokio::time::{Duration, interval};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "cloudflare")]
    platform: String,
    #[clap(short, long, default_value = "")]
    domain_name: String,

    #[clap(long, default_value = "")]
    zone: String,
    #[clap(long, default_value = "")]
    auth_email: String,
    #[clap(long, default_value = "")]
    auth_key: String,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    println!("命令行参数 {:?}", args);

    let mut wan_ip = String::new();
    let mut interval = interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let result = run(&args, &mut wan_ip).await;
        println!("执行时间:{},执行结果:{}", Local::now().format("%Y-%m-%d %H:%M:%S"), result.is_ok());
    }
}

async fn run(args: &Args, wan_ip: &mut String) -> Result<(), Box<dyn Error>> {
    //查询IP
    let client = reqwest::Client::new();
    let resp_json = client
        .get("https://httpbin.org/ip")
        .send().await?
        .json::<Value>().await?;

    let origin = resp_json["origin"].as_str().unwrap();
    if origin != wan_ip {
        *wan_ip = origin.to_string();
        //更新远程IP
        println!("当前的WAN地址为:{}", wan_ip);
        match args.platform.as_str() {
            "cloudflare" => {
                if args.zone.is_empty() || args.auth_email.is_empty() || args.auth_key.is_empty() || args.domain_name.is_empty() {
                    eprintln!("zone,auth_email,auth_key,domain_name不能为空");
                    return Ok(());
                }
                //先查询
                let url = &format!("https://api.cloudflare.com/client/v4/zones/{zone}/dns_records", zone = args.zone);
                let resp_json = client.get(url).header("Content-Type", "application/json")
                    .header("X-Auth-Email", &args.auth_email)
                    .header("X-Auth-Key", &args.auth_key)
                    .query(&[("name", &args.domain_name)])
                    .send().await?
                    .json::<Value>().await?;
                let result = resp_json["result"].as_array();
                let json_object = &json!({"type":"A","name": args.domain_name,"content": wan_ip,"ttl":1});
                let resp_json = if result.is_some() {
                    //编辑
                    println!("DNS设置(编辑)");
                    let id = result.unwrap().get(0).unwrap().as_object().unwrap().get("id").unwrap().as_str().unwrap();
                    client.put(format!("{}/{}", url, id))
                        .header("Content-Type", "application/json")
                        .header("X-Auth-Email", &args.auth_email)
                        .header("X-Auth-Key", &args.auth_key)
                        .json(json_object)
                        .send().await?
                        .json::<Value>().await?
                } else {
                    //新增
                    println!("DNS设置(新增)");
                    client.post(url)
                        .header("Content-Type", "application/json")
                        .header("X-Auth-Email", &args.auth_email)
                        .header("X-Auth-Key", &args.auth_key)
                        .json(json_object)
                        .send().await?
                        .json::<Value>().await?
                };
                println!("设置结果:{}", resp_json);
            }
            _ => {
                eprintln!("platform({})不支持", args.platform);
            }
        }
    };
    Ok(())
}