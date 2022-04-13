extern crate clap;
use clap::{Arg, App};
use std::io;
use std::io::{BufRead, BufReader};
use std::io::{stdout, Write};
use std::fs;
use std::fs::File;
use curl::easy::Easy;

/*
    the struct below is needed to organize elements for different http calls
    the information to initialize this struct  is included in the  .provider_details.cfg file and
    should be written in 'csv' form like this:
weatherapi,http://api.weatherapi.com/v1/current.json?,key,q
*/
struct ProviderDetails {
name: String, //
main_url: String,
id_field_name: String, //this field may have name 'id', 'key'  or other
addr_field_name: String, //feild name the value of which indicates geographic location
}


fn read_default_provider() -> String {
       let filename = ".provider.cfg"; //this file contains default provider name (user defined)
       let contents = fs::read_to_string(filename).unwrap_or("OpenWeather".to_string());
       return contents;
}

fn read_provider_id(provider: String) -> String
{
    let filename = ".provider_id.cfg";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let split = line.split(",");
        let vec: Vec<&str> = split.collect();
        if provider == vec[0].to_string() {return vec[1].to_string()};
    }
    return "not_found".to_string();
}


fn write_provider_id(provider: String, id: String)
{
    let filename = ".provider_id.cfg"; //csv file with providers' keys/ids (created at run time as a userd enters the data)
    let mut new_content = String::new();
    let mut written = false;
    let mut first_line = true;
    let file_exists = std::path::Path::new(filename).exists();
    if file_exists {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let split = line.split(",");
        let vec: Vec<&str> = split.collect();
        if provider == vec[0].to_string() {
            if first_line { first_line = false;
            } else { new_content.push_str("\n"); }

            new_content.push_str(vec[0]);
            new_content.push_str(",");
            new_content.push_str(&id[..]);
            written = true;
        } else {
            if first_line { first_line = false;
            } else { new_content.push_str("\n"); }

            new_content.push_str(&line[..]);
        }
    }
    }
    if ! written {
            if ! first_line { new_content.push_str("\n"); }

            new_content.push_str(&provider[..]);
            new_content.push_str(",");
            new_content.push_str(&id[..]);
    }
    let mut file = std::fs::File::create(filename).expect("create failed");
    file.write_all(new_content.as_bytes()).expect("write failed");
    println!("Id for provider {} updated", provider );
}

fn write_default_provider(provider: &str)
{
    let filename = ".provider.cfg";
    let mut file = std::fs::File::create(filename).expect("create failed");
    file.write_all(provider.as_bytes()).expect("write failed");
    println!("updated default provider" );
}

fn read_providers_names() -> Vec<String>
{
    let mut v = Vec::new();
    let filename = ".provider_details.cfg";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let split = line.split(",");
        let vec: Vec<&str> = split.collect();
        v.push(vec[0].to_string());
    }
    return v;
}

fn read_provider_details(provider: String) -> ProviderDetails
{
    let filename = ".provider_details.cfg";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    for (_index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let split = line.split(",");
        let vec: Vec<&str> = split.collect();
        if provider == vec[0].to_string() {
            return
            ProviderDetails {
                name:            vec[0].to_string(),
                main_url:        vec[1].to_string(),
                id_field_name:   vec[2].to_string(),
                addr_field_name: vec[3].to_string(),
            }
        };
    }
    return
    ProviderDetails {
        name:            "not_found".to_string(),
        main_url:        String::new(),
        id_field_name:   String::new(),
        addr_field_name: String::new(),
    }
}

fn compose_url(details: ProviderDetails, id: String, address: String) -> String {
    let v = vec![details.main_url, details.id_field_name, "=".to_string(),
                 id,"&".to_string(), details.addr_field_name, "=".to_string(), address ];
    return v.join("");
}

fn provider_config(provider: String) {
    let mut id = String::new();
    io::stdin().read_line(&mut id).expect("Failed to read line");
    if id.ends_with("\n") {id.pop();}
    write_provider_id(provider, id);
}

fn main() {
    let matches = App::new("CLI Weather application")
        .version("0.1.0")
        .author("Weather Ukraine 2022")
        .about("CLI to get weather from preconfigured providers")
        .arg(Arg::with_name("provider_to_configure")
                 .short("c")
                 .long("configure")
                 .takes_value(true)
                 .help(concat!("choose a provider to be configured.\nExample:\n",
                               "./weather_app configure OpenWeather")))
        .arg(Arg::with_name("new_default_provider")
                 .short("d")
                 .long("setdefault")
                 .takes_value(true)
                 .help(concat!("set a default provider.\nExample:\n",
                               "./weather_app --setdefault OpenWeather")))
        .arg(Arg::with_name("address")
                 .short("g")
                 .long("get")
                 .takes_value(true)
                 .help(concat!("Get weather for your city.\nExample:\n",
                               "./weather_app --get Zhytomyr")))
        .arg(Arg::with_name("provider")
                 .short("p")
                 .long("provider")
                 .takes_value(true)
                 .help(concat!("which provider to use.\nExample\n",
                               "./weather_app --get Zhytomyr --provider OpenWeather")))
        .get_matches();

    let new_default = matches.value_of("new_default_provider");
    match new_default {
        None => {}
        Some(s) => {
            let providers_names = read_providers_names();
            if providers_names.contains(&s.to_string()) {
                 println!("Setting new default provider: {}", s);
                write_default_provider(s);
            } else {
              println!("Unknown provider {}", s);
              print!("Possible rpoviders:");
              for p in providers_names {print!(" {}",p);}
              println!("\n");
            }
        return;
        }
    }

    let provider = matches.value_of("provider_to_configure");
    match provider {
        None => {}
        Some(s) => {
            let providers_names = read_providers_names();
            if providers_names.contains(&s.to_string()) {
                 println!("Enter an Id for provider: {}", s);
                 provider_config(s.to_string());
                 return;
            } else {
              println!("Unknown provider {}", s);
              print!("Possible rpoviders:");
              for p in providers_names {print!(" {}",p);}
              println!("\n");
              return;
            }

        }
    }

//reading address with -- get address
    let address = matches.value_of("address");
    match address {
        None => {} //println!("No address indicated."),
        Some(city) => {
            let mut provider = read_default_provider();
            let prvdr = matches.value_of("provider");
            match prvdr {
                None => println!("No provider indicated."),
                Some(prov) => {
                    let providers_names = read_providers_names();
                    if providers_names.contains(&prov.to_string()) {
                        provider = prov.to_string();
                    }
                }
            }
            let id = read_provider_id(provider.clone());
            let details = read_provider_details(provider);
            println!("Using provider {}", details.name);
            let url = compose_url(details, id, city.to_string());

            let mut easy = Easy::new();
            easy.url(&url[..]).unwrap();
            println!("Current weather for {}", city);
            easy.write_function(|data| {
                stdout().write_all(data).unwrap();
                Ok(data.len())
            }).unwrap();
            easy.perform().unwrap();
            print!("\n");

        return;
        }
    }
    println!("Use corresponding commands and options to save ids/keys for weather providers and to get current weather.");
    println!("To get help, run\n./weather_app --help");
}
