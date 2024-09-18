#![allow(non_snake_case)]
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::str::from_utf8;
use std::collections::HashSet;

use reqwest::Client;
use clap::{Arg, AppSettings, App};
use tokio::runtime;
use futures::stream::{self, StreamExt};
use kuchiki::traits::*;
use url::Url;
use psl::{List, Psl};

struct DomainResult {
    unit: String,
    type_: String,
    icp_code: String,
    domain: String,
    pass_time: String,
}

fn main() {
    create_file_if_not_exists("result.txt");
    let matches = App::new("ICP Lookup Tool")
        .setting(AppSettings::ArgRequiredElseHelp)
        .author("Author: Bob ;(")
        .about("Tool for querying ICP filings by domain name or company name or url")
        .arg(Arg::with_name("domain")
            .short('d')
            .long("domain")
            .value_name("DOMAIN")
            .takes_value(true)
            .help("Domain name Or Company name Or URL to lookup"))
        .arg(Arg::with_name("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .takes_value(true)
            .help("A file containing the domain or business name or url to be found"))
        .get_matches();

    let runtime = runtime::Runtime::new().unwrap();

    if let Some(domain) = matches.value_of("domain") {
        let url = build_url_xpath(domain);
        // println!("{}", url);
        match runtime.block_on(fetch_and_handle_data_xpath(&url)) {
            Ok(_) => println!("Data processing completed."),
            Err(err) => println!("Error: {}", err)
        };
    } else if let Some(filename) = matches.value_of("file") {
        match runtime.block_on(process_file(filename)) {
            Ok(_) => println!("Data processing completed."),
            Err(err) => println!("Error: {}", err)
        };
    } else {
        eprintln!("Invalid option.");
    }
}


fn get_uuid() -> String {
    let uuid_template = "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx";
    let mut uuid = String::new();
    for c in uuid_template.chars() {
        let v = match c {
            'x' => rand::random::<u32>() % 16,
            'y' => ((rand::random::<u32>() % 16) & 0x3) | 0x8,
            '4' => 4,
            '-' => u32::MAX, // 使用一个特殊值表示连字符
            _ => {
                assert!(false, "Unexpected character in UUID template: {}", c);
                0 // 这个值不会被使用，因为 assert! 会 panic
            }
        };
        if v == u32::MAX {
            uuid.push('-');
        } else {
            uuid.push_str(&format!("{:x}", v));
        }
    }
    uuid
}

fn get_root_domain(input: &str) -> Option<String> {
    let domain_str = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else {
        format!("http://{}", input)
    };
    
    let parsed_url = Url::parse(&domain_str).ok()?;
    let host = parsed_url.host_str()?;

    let list = List;
    let suffix = list.suffix(host.as_bytes())?;

    let suffix_byte = suffix.as_bytes();
    let suffix_str = from_utf8(suffix_byte).unwrap();

    let domain = host.trim_end_matches(suffix_str).trim_end_matches('.');
    let parts: Vec<&str> = domain.rsplitn(2, '.').collect();

    parts.first().map(|last| format!("{}.{}", last, suffix_str))
}

fn contains_chinese(s: &str) -> bool {
    for ch in s.chars() {
        if !ch.is_ascii() {
            return true;
        }
    }
    false
}

fn build_url_xpath(input: &str) -> String {
    let index =  if !contains_chinese(input) {get_root_domain(input).expect("Failed to get root domain")
    } else {
        input.to_string()
    };
    format!(
        "https://www.beianx.cn/search/{}",
        index
    )
}

async fn fetch_and_handle_data_xpath(url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let data = fetch_data(url).await?;
    handle_data_xpath(&data);
    Ok(())
}

async fn fetch_data(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let uuid = get_uuid();
    let cookie_str = format!("machine_str={}", uuid);
    let client = Client::new();
    let response = client.get(url).header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36").header("Cookie", cookie_str).send().await?;
    let body = response.text().await?;
    Ok(body)
}

fn create_file_if_not_exists(file_path: &str) {
    let path = Path::new(file_path);
    if !path.exists() {
        File::create(path).expect("Failed to create file");
    }
}

fn process_domain_result(data_in_row: &Vec<String>, file: &mut File) {
    let result = DomainResult {
        unit: data_in_row[1].clone(),
        type_: data_in_row[2].clone(),
        icp_code: data_in_row[3].clone(),
        domain: data_in_row[data_in_row.len() - 4].clone() ,
        pass_time: data_in_row[data_in_row.len() - 3].clone(),
    };

    let output = format!("[Unit]: {} [Type]: {} [icpCode]: {} [Domain]: {} [passTime]: {}", &result.unit, &result.type_, &result.icp_code, &result.domain, &result.pass_time);

    println!("{}", output);

    if let Err(e) = writeln!(file, "{}", output) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn handle_data_xpath(data: &str) {
    let document = kuchiki::parse_html().one(data);
    let css_selector = "table tbody tr";

    let selections: Vec<_> = document.select(css_selector).unwrap().collect();

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("result.txt")
        .unwrap();

    for tr in selections {
        let data_in_row: Vec<_> = tr.text_contents().split_whitespace().map(|s| s.to_owned()).collect();
        // println!("{:?}", data_in_row);
        if data_in_row.len() >= 8 {
            process_domain_result(&data_in_row, &mut file);
        } else {
            eprintln!("[Error] IPC filing query failed! Skipping!");

        }
    }
}

async fn process_file(filename: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let path = Path::new(filename);
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let urls: Vec<String> = reader.lines()
        .map_while(Result::ok)
        .map(|line| build_url_xpath(&line))
        .collect();

    let unique_urls: HashSet<String> = urls.into_iter().collect();
    let urls_set: Vec<String> = unique_urls.into_iter().collect();

    let fetches = urls_set.iter()
        .map(|url| fetch_and_handle_data_xpath(url));

    stream::iter(fetches)
        .buffer_unordered(50)
        .for_each(|result| async { 
            if let Err(e) = result {
                // 打印错误信息
                println!("Error: {}", e);
            }
        })
        .await;

    Ok(())
}
