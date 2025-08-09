// use std::process::exit;
// use anyhow::{anyhow};
mod command_parser;
mod upyun;
mod upyun_common;

/*
../target/debug/upyun_tool --download /music-room/5f4cfcbb-cfd0-4f9c-b4ab-0b96cd7b0583.mp4 5f4cfcbb-cfd0-4f9c-b4ab-0b96cd7b0583.mp4 --server 1
../target/debug/upyun_tool --upload /music-room/5f4cfcbb-cfd0-4f9c-b4ab-0b96cd7b0583.mp4 5f4cfcbb-cfd0-4f9c-b4ab-0b96cd7b0583.mp4 --server 1
../target/debug/upyun_tool --delete /music-room/5f4cfcbb-cfd0-4f9c-b4ab-0b96cd7b0583.mp4 --server 1
../target/debug/upyun_tool --upt-download https://upyun.alchemy-studio.cn/music-room/17c832cb-bc6b-41f7-96cd-190b7fe4a3d7.jpeg
 */

fn main() -> Result<(), anyhow::Error> {
    /* Test */

    /* Upyun rest download test pass */
    // let upyun = upyun::Upyun::new(".upyun_pass".to_string());
    // upyun.download("1dd6080a-f9b0-4831-94dd-9f26491cc8df.mp4".to_string(),
    //           "/music-room/1dd6080a-f9b0-4831-94dd-9f26491cc8df.mp4".to_string());

    /* Upyun rest upload test pass */
    // let upyun = upyun::Upyun::new(".upyun_pass".to_string());
    // upyun.upload("1dd6080a-f9b0-4831-94dd-9f26491cc8df.mp4".to_string(),
    //           "/music-room/1dd6080a-f9b0-4831-94dd-9f26491cc8df.mp4".to_string());

    /* Upyun upt test pass */
    // upyun::upyun_upt_download("https://upyun.alchemy-studio.cn/music-room/17c832cb-bc6b-41f7-96cd-190b7fe4a3d7.jpeg",
    //              ".upt_config");

    /* Release */

    command_parser::upyun_tools_command_parser()

    // Ok(())
}
