use crypto::digest::Digest;
use crypto::md5::Md5;
use data_encoding::BASE64;
use std::fs::File;
use std::io::Cursor;
use std::io::{BufRead, BufReader};
// use reqwest::Response;
use ring::hmac;
// use anyhow::anyhow;
use crate::upyun_common;
use anyhow::anyhow;
use std::path::Path;
use std::time::Duration;
use url::Url;

#[derive(Debug)]
pub struct Upyun {
    pub bucket: String,
    pub operator: String,
    pub password: String,
}

impl Upyun {
    pub fn new(config_file: String) -> Self {
        /* Read upyun config from .upyun_pass */
        let upyun_pass_file = config_file;
        let upyun_pass_file = match File::open(&upyun_pass_file) {
            Err(error_message) => {
                panic!("Couldn't not open {} : {}", upyun_pass_file, error_message)
            }
            Ok(upyun_pass_file) => upyun_pass_file,
        };

        /* Read upyun config file to buffer*/
        let mut buffered_file = BufReader::new(upyun_pass_file);
        assert!(buffered_file.buffer().is_empty());

        /* Upyun operator */
        let mut operator = String::new();
        let mut _num_bytes = buffered_file
            .read_line(&mut operator)
            .expect("reading from cursor won't fail");
        operator = operator.as_str().trim().to_string();
        println!("User : {} length : {}", operator, operator.len());

        /* Upyun password */
        let mut password = String::new();
        _num_bytes = buffered_file
            .read_line(&mut password)
            .expect("reading from cursor won't fail");
        password = password.as_str().trim().to_string();
        println!("Passwd : {} length : {}", password, password.len());

        /* Upyun bucket */
        let mut bucket = String::new();
        _num_bytes = buffered_file
            .read_line(&mut bucket)
            .expect("reading from cursor won't fail");
        bucket = bucket.as_str().trim().to_string();
        println!("Bucket : {} length : {}", bucket, bucket.len());

        Self {
            bucket,
            operator,
            password,
        }
    }

    pub fn upload(&self, local_file: String, remote_file: String, server_index: i32) -> Result<(), anyhow::Error> {
        let gmt_date = upyun_common::get_gmt_date();
        let date = gmt_date.as_str();
        let signature = self.signature("PUT", remote_file.as_str(), date);

        /* Upyun domain */
        // let base_url = "https://v1.api.upyun.com";
        let base_url = format!("https://v{}.api.upyun.com", server_index);

        /* Create client */
        let client = reqwest::blocking::Client::new();

        /* Construct request url */
        let request_url = base_url.to_string() + "/" + self.bucket.as_str() + "/" + remote_file.as_str();
        println!("Request url : {}", request_url);

        let file = match File::open(local_file) {
            Ok(file) => file,
            Err(err) => panic!("Problem open the file: {:?}", err),
        };

        let resp = client
            .put(request_url)
            .header("Authorization", signature)
            .header("Date", gmt_date)
            .body(file)
            .timeout(Duration::from_secs(5000))
            .send()?;

            println!("-= Response start =-");
            println!("{:?}", resp);
            println!("-= Response end =-");

            if resp.status() == reqwest::StatusCode::OK {
                Ok(())
            } else {
                println!("Response status code: {}", resp.status());
                Err(anyhow!(resp.status()))
            }
    }

    pub fn download(&self, local_file: String, remote_file: String, server_index: i32) -> Result<(), anyhow::Error> {
        let gmt_date = upyun_common::get_gmt_date();
        let date = gmt_date.as_str();
        let signature = self.signature("GET", remote_file.as_str(), date);

        /* Upyun domain */
        // let base_url = "https://v1.api.upyun.com";
        let base_url = format!("https://v{}.api.upyun.com", server_index);

        /* Create client */
        let client = reqwest::blocking::Client::new();

        /* Construct request url */
        let request_url = base_url.to_string() + "/" + self.bucket.as_str() + "/" + remote_file.as_str();
        println!("Request url : {}", request_url);

        let resp = client
            .get(request_url)
            .header("Authorization", signature)
            .header("Date", gmt_date)
            .send()?;

        println!("-= Response start =-");
        println!("{:?}", resp);
        println!("-= Response end =-");

        if resp.status() == reqwest::StatusCode::OK {
            let mut file = std::fs::File::create(local_file)?;
            let mut content = Cursor::new(resp.bytes()?);
            std::io::copy(&mut content, &mut file)?;
            Ok(())
        } else {
            println!("Response status code: {}", resp.status());
            Err(anyhow!(resp.status()))
        }
    }

    pub fn delete(&self, remote_file: String, server_index: i32) -> Result<(), anyhow::Error> {
        let gmt_date = upyun_common::get_gmt_date();
        let date = gmt_date.as_str();
        let signature = self.signature("DELETE", remote_file.as_str(), date);

        /* Upyun domain */
        // let base_url = "https://v1.api.upyun.com";
        let base_url = format!("https://v{}.api.upyun.com", server_index);

        /* Create client */
        let client = reqwest::blocking::Client::new();

        /* Construct request url */
        let request_url = base_url.to_string() + "/" + self.bucket.as_str() + "/" + remote_file.as_str();
        println!("Request url : {}", request_url);

        let resp = client
            .delete(request_url)
            .header("Authorization", signature)
            .header("Date", gmt_date)
            .send()?;

        println!("-= Response start =-");
        println!("{:?}", resp);
        println!("-= Response end =-");

        if resp.status() == reqwest::StatusCode::OK {
            Ok(())
        } else {
            println!("Response status code: {}", resp.status());
            Err(anyhow!(resp.status()))
        }
    }

    fn signature(&self, method: &str, uri: &str, date: &str) -> String {
        let bucket = self.bucket.as_str();
        let operator = self.operator.as_str();
        let password = self.password.as_str();

        /* Debug */
        println!("Debug - bucket : {}", bucket);
        println!("Debug - operator : {}", operator);
        println!("Debug - password : {}", password);
        println!("Debug - method : {}", method);
        println!("Debug - uri : {}", uri);
        println!("Debug - date : {}", date);

        /* MD5 password */
        let mut md5 = Md5::new();
        md5.input(password.as_bytes());
        let md5_password = md5.result_str();
        println!("MD5 password : {}", md5_password);

        /* HMAC key */
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, md5_password.as_bytes());
        /* Concat method + uri + date + (content-md5) */
        let request_parameters = method.to_string() + "&" + "/" + bucket + "/" + uri + "&" + date;

        println!("Request parameters : {}", request_parameters);

        /* HMAC-SHA1 */
        let hmac_sha1 = hmac::sign(&hmac_key, request_parameters.as_bytes());

        /* Upyun authorization */
        let upyun_authorization = BASE64.encode(hmac_sha1.as_ref());

        format!("UPYUN {}:{}", operator, upyun_authorization)
    }
}

pub fn calc_upt(upt_secret: &str, upt_duration: u64, uri: &str) -> String {
    println!("Debug - secret : {}", upt_secret);
    println!("Debug - duration : {}", upt_duration);
    println!("Debug - uri : {}", uri);

    /* End time*/
    let end_time = upyun_common::get_sys_time_in_secs() + upt_duration;
    // let end_time = 1665670877; //Test
    println!("End time : {}", end_time);

    /* MD5 */
    let md5_string = format!("{}&{}&{}", upt_secret, end_time, uri);
    println!("MD5 string : {}", md5_string);
    let mut md5 = Md5::new();
    md5.input(md5_string.as_bytes());
    let md5_upt = md5.result_str();
    println!("Signature : {}", md5_upt);

    /* Upt */
    let upt_sub_string = &md5_upt[12..20];
    let upt_string = format!("{}{}", upt_sub_string, end_time);
    println!("Upt string : {}", upt_string);

    upt_string
}

pub fn upyun_upt_download(url: &str, config_file: &str) -> Result<(), anyhow::Error> {
    let issue_list_url = Url::parse(url)?;

    let uri = issue_list_url.path();

    println!("Upt url : {}", url);
    println!("Upt uri : {}", uri);

    /* Read upt config from config file */
    let upt_config_file = config_file;
    let upt_config_file = match File::open(&upt_config_file) {
        Err(error_message) => panic!("Couldn't not open {} : {}", upt_config_file, error_message),
        Ok(upt_config_file) => upt_config_file,
    };

    /* Read upt config file to buffer*/
    let mut buffered_file = BufReader::new(upt_config_file);
    assert!(buffered_file.buffer().is_empty());

    /* upt secret */
    let mut secret = String::new();
    let mut _num_bytes = buffered_file
        .read_line(&mut secret)
        .expect("reading from cursor won't fail");
    secret = secret.as_str().trim().to_string();
    println!("Secret : {} length : {}", secret, secret.len());

    /* upt duration */
    let mut duration = String::new();
    _num_bytes = buffered_file
        .read_line(&mut duration)
        .expect("reading from cursor won't fail");
    duration = duration.as_str().trim().to_string();
    println!("Duration : {} length : {}", duration, duration.len());
    let duration_time = duration.parse::<u64>()?;
    println!("Duration : {}", duration_time);

    /* Construct request url  */
    let upt = calc_upt(secret.as_str(), duration_time, uri);
    let request_url = url.to_string() + "?_upt=" + &*upt;
    println!("Upt request url : {}", request_url);

    /* Create client */
    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(request_url)
        .timeout(Duration::from_secs(5000))
        .send()?;

    println!("{:#?}", resp);

    if resp.status() == reqwest::StatusCode::OK {
        let path = Path::new(uri);
        let local_file = path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Failed to get file name from path"))?;
        let mut file = std::fs::File::create(local_file)?;
        let mut content = Cursor::new(resp.bytes()?);
        std::io::copy(&mut content, &mut file)?;

        Ok(())
    } else {
        println!("Response status code: {}", resp.status());
        Err(anyhow!(resp.status()))
    }
}
