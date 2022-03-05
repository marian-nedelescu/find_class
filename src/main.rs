use std::fs;
use std::fs::metadata;
use std::fs::File;
use std::io::{Cursor, Read, Seek};

use clap::Parser;
use itertools::Itertools;

fn print_zip_item<R: Read + Seek>(prefix: &str, input: R) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut zip = zip::ZipArchive::new(input).unwrap();
    let file_names_list = zip
        .file_names()
        .map(|item| item.to_owned())
        .collect::<Vec<String>>();
    for name in file_names_list {
        let mut file = zip.by_name(name.as_str()).unwrap();
        let is_file = file.is_file();
        if is_file {
            result.push(format!("{} ::: {}", prefix, name));
            if name.ends_with(".jar") || name.ends_with(".war") {
                let mut buf: Vec<u8> = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                let c = Cursor::new(buf);
                result.extend(print_zip_item(result.last().unwrap(), c));
            }
        }
    }
    result
}

fn show_files(dir: &str) -> Vec<String> {
    let zip_types = vec!["zip", "jar", "ear", "war"];
    let handle_file = |file_name: &str| -> Vec<String> {
        let pos = file_name.rfind('.');
        if let Some(position) = pos {
            let suffix = file_name[position + 1..].to_owned();
            if zip_types.contains(&suffix.as_str()) {
                let file = File::open(&file_name).unwrap();
                print_zip_item(file_name, &file)
            } else {
                vec![file_name.to_string()]
            }
        } else {
            vec![file_name.to_string()]
        }
    };
    let handle_dir = |dir: &str| {
        fs::read_dir(dir)
            .unwrap()
            .map(|res| res.map(|e| e.path()).unwrap())
            .flat_map(|item| {
                let file_name = item
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .as_str()
                    .to_owned();
                show_files(file_name.as_str())
            })
            .collect_vec()
    };
    if metadata(dir).unwrap().is_dir() {
        handle_dir(dir)
    } else {
        handle_file(dir)
    }
}

#[derive(Parser)]
struct Cli {
    /// The path where look for the filter
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,

    /// The filter
    #[clap(short, long)]
    filter: String,
}

fn main() {
    let args = Cli::parse();
    let l = show_files(args.path.as_os_str().to_str().unwrap());
    l.iter()
        .filter(|name| name.contains(args.filter.as_str()))
        .for_each(|item| println!("Item :{}", item));
}
