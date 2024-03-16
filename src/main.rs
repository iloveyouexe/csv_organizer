use clap::{App, Arg};
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("CSV Organizer")
        .version("0.1.0")
        .author("bmo")
        .about("Organizes CSV files into specific folders based on their headers.")
        .arg(
            Arg::with_name("directory")
                .short('d')
                .long("directory")
                .value_name("DIRECTORY")
                .help("Sets the input directory with CSV files")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let directory = matches.value_of("directory").unwrap();

    visit_dirs(Path::new(directory), &organize_file)?;

    // Move remaining files to Uncategorized directory
    move_remaining_files(Path::new(directory))?;

    Ok(())
}

// add more later
fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn organize_file(entry: &DirEntry) {
    let file_path = entry.path();
    println!("Processing file: {:?}", file_path);

    if file_path.extension().map_or(false, |e| e == "csv") {
        let mut rdr = csv::Reader::from_path(&file_path).expect("Failed to open CSV...");
        let headers = rdr.headers().expect("Failed to read headers...");

        match determine_file_type(headers) {
            Some(FileType::Product) => {
                if let Err(err) = copy_file(&file_path, "Products") {
                    eprintln!("Error moving file: {:?}", err);
                    move_to_uncategorized(&file_path);
                }
            }
            Some(FileType::ProductCosts) => {
                if let Err(err) = copy_file(&file_path, "ProductCosts") {
                    eprintln!("Error moving file: {:?}", err);
                    move_to_uncategorized(&file_path);
                }
            }
            None => {
                println!("Unknown or unsupported CSV type: {:?}", file_path);
                move_to_uncategorized(&file_path);
            }
        }
    }
}

use std::collections::HashSet;

fn determine_file_type(headers: &csv::StringRecord) -> Option<FileType> {
    let lowercase_headers: HashSet<_> = headers
        .iter()
        .map(|h| h.to_lowercase())
        .collect();

    let product_headers: HashSet<String> = ["productname".to_string()].iter().cloned().collect();
    let cost_headers: HashSet<String> = ["cost".to_string()].iter().cloned().collect();

    if !lowercase_headers.is_disjoint(&product_headers) {
        Some(FileType::Product)
    } else if !lowercase_headers.is_disjoint(&cost_headers) {
        Some(FileType::ProductCosts)
    } else {
        None
    }
}

enum FileType {
    Product,
    ProductCosts,
}

fn copy_file(file_path: &Path, destination_dir: &str) -> std::io::Result<()> {
    let source_file_name = file_path.file_name().unwrap();
    let destination_path = file_path.parent().unwrap().join(destination_dir).join(source_file_name);

    if !destination_path.parent().unwrap().exists() {
        fs::create_dir_all(destination_path.parent().unwrap())?;
    }

    fs::copy(file_path, &destination_path)?;

    Ok(())
}

fn move_to_uncategorized(file_path: &Path) {
    let destination_dir = file_path.parent().unwrap().join("UncategorizedBackups");

    if !destination_dir.exists() {
        fs::create_dir(&destination_dir).unwrap_or_else(|_| {
            panic!("Failed to create directory: {:?}", destination_dir);
        });
    }

    if let Err(err) = fs::rename(&file_path, destination_dir.join(file_path.file_name().unwrap())) {
        eprintln!("Error moving file to UncategorizedBackups: {:?}", err);
    }
}

fn move_remaining_files(directory: &Path) -> io::Result<()> {
    let uncategorized_dir = directory.join("Uncategorized");

    if !uncategorized_dir.exists() {
        fs::create_dir(&uncategorized_dir)?;
    }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_file() && file_path.extension().map_or(false, |e| e == "csv") {
            if let Err(err) = fs::rename(&file_path, uncategorized_dir.join(file_path.file_name().unwrap())) {
                eprintln!("Error moving file to Uncategorized: {:?}", err);
            }
        }
    }

    Ok(())
}
