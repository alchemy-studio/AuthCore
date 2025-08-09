use anyhow::anyhow;
use crate::upyun::*;
use clap::Parser;

#[derive(Parser)]
struct UpyunToolsCli {
    #[arg(
    long,
    group = "actions",
    group = "rest_actions",
    requires = "remote",
    requires = "local"
    )]
    upload: bool,

    #[arg(
    long,
    group = "actions",
    group = "rest_actions",
    requires = "remote",
    requires = "local"
    )]
    download: bool,

    #[arg(long, group = "actions", group = "rest_actions", requires = "remote")]
    delete: bool,

    #[arg(long, group = "actions", group = "upt_actions", requires = "remote")]
    upt_download: bool,

    #[arg(group = "remote", requires = "actions")]
    remote_file_string: Option<String>,

    #[arg(group = "local", requires = "rest_actions")]
    local_file_string: Option<String>,

    #[arg(short, long, requires = "actions")]
    config: Option<String>,

    #[arg(short, long, requires = "actions")]
    server: Option<i32>,
}

pub fn upyun_tools_command_parser() -> Result<(), anyhow::Error> {
    let cli = UpyunToolsCli::parse();

    println!("Command value for upload: {}", cli.upload);
    let upload = cli.upload;

    println!("Command value for download: {}", cli.download);
    let download = cli.download;

    println!("Command value for delete: {}", cli.delete);
    let delete = cli.delete;

    println!("Command value for upt_download: {}", cli.upt_download);
    let upt_download = cli.upt_download;

    let remote_file_string = cli.remote_file_string.as_deref().map(|s| s.to_string()).unwrap_or_default();
    if !remote_file_string.is_empty() {
        println!("Command value for remote file: {}", remote_file_string);
    }

    let local_file_string = cli.local_file_string.as_deref().map(|s| s.to_string()).unwrap_or_default();
    if !local_file_string.is_empty() {
        println!("Command value for local file: {}", local_file_string);
    }

    let config_file;
    if let Some(config) = cli.config.as_deref() {
        println!("Command value for config: {}", config);
        config_file = config;
    } else {
        if upload || download || delete {
            config_file = ".upyun_pass";
        } else if upt_download {
            config_file = ".upt_config";
        } else {
            config_file = "";
        }
    }

    let server_index = cli.server.unwrap_or(0);
    if cli.server.is_some() {
        println!("Command value for server: {}", server_index);
    }

    /* Debug */
    println!(" ---=== Debug ===--- ");
    println!("Debug : upload = {}", upload);
    println!("Debug : download = {}", download);
    println!("Debug : delete = {}", delete);
    println!("Debug : upt_download = {}", upt_download);
    println!("Debug : remote_file_string = {}", remote_file_string);
    println!("Debug : local_file_string = {}", local_file_string);
    println!("Debug : config_file = {}", config_file);
    println!("Debug : server_index = {}", server_index);

    /* Upyun rest */
    let upyun = Upyun::new(config_file.to_string());

    if upload {
        match upyun.upload(local_file_string, remote_file_string, server_index) {
            Ok(ok) => {
                println!("Upyun upload success .");
                Ok(ok)
            }
            Err(err) => {
                println!("Upyun upload failed !");
                Err(err)
            }
        }
    } else if download {
        match upyun.download(local_file_string, remote_file_string, server_index) {
            Ok(ok) => {
                println!("Upyun download success .");
                Ok(ok)
            }
            Err(err) => {
                println!("Upyun download failed !");
                Err(err)
            }
        }
    } else if delete {
        match upyun.delete(remote_file_string, server_index) {
            Ok(ok) => {
                println!("Upyun delete success .");
                Ok(ok)
            }
            Err(err) => {
                println!("Upyun delete failed !");
                Err(err)
            }
        }
    } else if upt_download {
        match upyun_upt_download(&remote_file_string, config_file) {
            Ok(ok) => {
                println!("Upt download success .");
                Ok(ok)
            }
            Err(err) => {
                println!("Upt download failed !");
                Err(err)
            }
        }
    } else {
        print!("Action should be <upload/download/delete/upt_download> !");
        Err(anyhow!("Input action error !"))
    }
}
