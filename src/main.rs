use std::fs;
use std::fs::{File};
use std::io;
use std::io::{BufReader, Error, Read, Seek};
use std::process::exit;

use clap::{App, Arg, ArgMatches};
use zip::ZipArchive;

use crate::traits::KotlinAny;

mod traits;

fn main() {
    let arguments = argument_matching();

    let verbose = arguments.is_present(ARG_FLAG_VERBOSE);

    let file_names = arguments.value_of(ARG_INPUT_FILE).or(arguments.value_of(ARG_INPUT_FILE_NAME))
        .map(|file_name| vec![file_name.to_string(), format!("{}.zip", file_name.to_string())])
        .unwrap();

    let archive = file_names.iter()
        .find_map(|file_name| {
            if verbose { println!("Looking for archive: {}", file_name) }
            File::open(file_name).ok()
                .map_or(None, |file| file.metadata().ok().map_or(None, |metadata| if metadata.is_file() { Some(file) } else { None }))
                .also(|file| if verbose && file.is_some() { println!("Processing archive: {}", file_name) })
        })
        .map(|file| BufReader::new(file))
        .map(|reader| ZipArchive::new(reader))
        .unwrap_or_else(exit_erroneous(format!("Unable to open '{}'", file_names.first().unwrap())))
        .unwrap();

    let project_file_names = arguments.values_of(ARG_PROJECT_FILES).unwrap().collect::<Vec<_>>();
    let project_file_name = archive.file_names()
        .find(|file_name| project_file_names.iter().any(|project_file_name| file_name.ends_with(project_file_name)))
        .unwrap_or_else(exit_erroneous(format!("Archive does not contain any of the specified project files {:?}", project_file_names)));
    if verbose { println!("Project file located: {}", project_file_name) }

    if verbose { println!("Extracting archive") }
    extract(archive, verbose).unwrap_or_else(|_| {
        eprintln!("Failed to extract archive '{}'", file_names.first().unwrap());
        exit(ERROR)
    });
}

fn argument_matching() -> ArgMatches {
    App::new("Unzip and open")
        .about("Unzips an archive if it contains exactly one projectFile and open that projectfile")
        .arg(Arg::new(ARG_INPUT_FILE)
            .about("Input file")
            .required_unless_present(ARG_INPUT_FILE_NAME)
            .index(1)
        )
        .arg(Arg::new(ARG_INPUT_FILE_NAME)
            .about("Input file")
            .required_unless_present(ARG_INPUT_FILE)
            .short('i')
            .long("input")
            .takes_value(true)
        )
        .arg(Arg::new(ARG_OUTPUT_DIR)
            .about("Output directory")
            .short('o')
            .long("output")
            .takes_value(true)
        )
        .arg(Arg::new(ARG_PROJECT_FILES)
            .about("Project file names to check for")
            .short('p')
            .long("projectFiles")
            .multiple_occurrences(true)
            .takes_value(true)
            .default_values(SPRING_DEFAULT_PROJECT_FILES)
            .required(true)
        )
        .arg(Arg::new(ARG_OPEN_WITH)
            .about("Open with command [system default if not set]")
            .short('w')
            .long("with")
            .takes_value(true)
        )
        .arg(Arg::new(ARG_FLAG_VERBOSE)
            .about("Be a little chatty")
            .short('v')
            .long("verbose")
        )
        .arg(Arg::new(ARG_FLAG_KEEP_SOURCE)
            .about("Keep source file")
            .short('k')
            .long("keepSource")
        )
        .get_matches()
}

fn extract<T: Read + Seek>(mut archive: ZipArchive<T>, verbose: bool) -> Result<(), Error> {
    for file_index in 0..archive.len() {
        let mut file = archive.by_index(file_index).unwrap();
        let file_name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        if verbose {
            file.comment()
                .take_if(|comment| !comment.is_empty())
                .map(|comment| println!("\tFile {} comment: {}", file_index, comment));
        }
        if file.is_dir() {
            if verbose { println!("\tFile {} extracted to \"{}\"", file_index, file_name.display()); }
            fs::create_dir_all(&file_name)?;
        } else {
            if verbose { println!("\tFile {} extracted to \"{}\" ({} bytes)", file_index, file_name.display(), file.size()); }
            file_name.parent()
                .map(|parent_path| if !parent_path.exists() {
                    fs::create_dir_all(&parent_path).unwrap();
                });
            let mut outfile = fs::File::create(&file_name)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn exit_erroneous<T>(message: String) -> Box<dyn Fn() -> T> {
    Box::new(move || {
        eprintln!("{}", message);
        exit(ERROR)
    })
}

const SPRING_DEFAULT_PROJECT_FILES: &[&str; 3] = &["build.gradle", "build.gradle.kts", "pom.xml"];
const ARG_INPUT_FILE: &'static str = "INPUT";
const ARG_INPUT_FILE_NAME: &'static str = "INPUT_FILE_NAME";
const ARG_PROJECT_FILES: &'static str = "PROJECT_FILE_NAME";
const ARG_OUTPUT_DIR: &'static str = "OUTPUT_DIR";
const ARG_OPEN_WITH: &'static str = "OPEN_WITH";
const ARG_FLAG_VERBOSE: &'static str = "VERBOSE";
const ARG_FLAG_KEEP_SOURCE: &'static str = "KEEP_SOURCE";
const ERROR: i32 = 1;
