use chrono::Local;
use clap::*;
use failure::Fail;
use regex::*;

use std::fmt::Display;
use std::fs;
use std::io;
use std::path::*;
use std::result::Result;

#[derive(Debug, Fail)]
pub enum NoterError {
    #[fail(display = "IO Error {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Error {}", _0)]
    Custom(failure::Error),
    #[fail(display = "Regex Error {}", _0)]
    RegexError(#[cause] regex::Error),
    #[fail(display = "Could not find notes folder for course {}", _0)]
    CourseNotFoundError(String),
}

impl From<io::Error> for NoterError {
    fn from(err: io::Error) -> NoterError {
        NoterError::IoError(err)
    }
}

impl From<failure::Error> for NoterError {
    fn from(err: failure::Error) -> NoterError {
        NoterError::Custom(err)
    }
}

impl From<regex::Error> for NoterError {
    fn from(err: regex::Error) -> NoterError {
        NoterError::RegexError(err)
    }
}

fn extract_param(param: &str, command: &ArgMatches<'_>) -> Option<String> {
    Some(String::from(command.args.get(param)?.vals[0].to_str()?))
}

fn find_course_path(root: &Path, course: &str) -> Result<PathBuf, NoterError> {
    let re = Regex::new(&format!(r"^({})\s.+", course))?;
    re.replace_all(course, "$course");
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        if !metadata.is_dir() {
            continue;
        }
        if re.is_match(
            path.file_name()
                .map_or_else(|| "", |s| s.to_str().unwrap_or("")),
        ) {
            return Ok(path);
        }
    }

    Err(NoterError::CourseNotFoundError(String::from(course)))
}

fn init_matches<'a>() -> ArgMatches<'a> {
    App::new("noter")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("new")
                .arg(Arg::with_name("course").required(true))
                .arg(Arg::with_name("title").required(false)),
        )
        .subcommand(
            SubCommand::with_name("course")
                .arg(Arg::with_name("code").required(true))
                .arg(Arg::with_name("title").required(true)),
        )
        .get_matches()
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<(), NoterError> {
    let matches = init_matches();
    match matches.subcommand() {
        ("new", Some(command)) => {
            // course_code should *always* be available, by clap
            let course_code = extract_param("course", command).unwrap();
            let title = extract_param("title", command);
            let mut path = find_course_path(std::env::current_dir()?.as_path(), &course_code)?;
            make_new_note(&mut path, &course_code, title.as_ref())?
        }
        ("course", Some(command)) => {
            // should always be available.
            let course_code = extract_param("code", command).unwrap();
            let title = extract_param("title", command).unwrap();
            let mut path = PathBuf::from(std::env::current_dir()?.as_path());
            make_new_folder(&mut path, &course_code, &title)?
        }
        _ => (),
    }
    Ok(())
}

fn make_new_folder<T: AsRef<str> + Display>(
    path: &mut PathBuf,
    course_code: T,
    title: T,
) -> Result<(), NoterError> {
    path.push(format!("{} {}", course_code, title));

    if !path.exists() {
        fs::create_dir(path)?;
        println!("Created folder for {} {}.", course_code, title);
    } else {
        println!("Folder for {} {} already exists.", course_code, title);
    }
    Ok(())
}

fn make_new_note<T: AsRef<str> + Display>(
    path: &mut PathBuf,
    course_code: T,
    title: Option<T>,
) -> Result<(), NoterError> {
    let date = format!("{}", Local::today().format("%F"));

    let new_file = title.map_or(format!("{}.md", date), |title| {
        format!("{}-{}.md", date, title)
    });

    path.push(&new_file);
    if path.exists() {
        println!("{}::{} already exists.", course_code, &new_file);
        return Ok(());
    }
    fs::File::create(&path)?;
    println!("Created {}::{}", course_code, &new_file);
    Ok(())
}
