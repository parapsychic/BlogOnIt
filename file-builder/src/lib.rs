use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io,
    path::Path,
    time::SystemTime,
};

use parser_generator::{
    parse_and_generator,
    parser::markdown_elements::{Link, Text},
    utils::remove_all_special,
};

pub fn start(folder_path: &str) -> Result<(), String> {
    let start_path = Path::new(folder_path);
    if !start_path.is_dir() {
        return Err(format!(
            "Cannot access directory: {}",
            start_path.to_str().unwrap()
        ));
    }

    match create_directory(start_path, &build_html_from_file) {
        Ok((files, pages)) => {
            create_index(files, pages).unwrap();
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    Ok(())
}

fn build_html_from_file(entry: &DirEntry) -> io::Result<(Text, Link)> {
    if entry.path().file_name().unwrap() == "index.md" {
        return Ok(("Home".to_string(), "/".to_string()));
    }
    let path = entry.path();
    let parent = path.parent().expect("");
    let file = format!(
        "{}",
        remove_all_special(
            entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split(".")
                .next()
                .unwrap()
        )
    );

    fs::create_dir_all(format!("website/{}", parent.to_str().unwrap()))?;

    let md = fs::read_to_string(entry.path()).expect("Cannot read file");
    let (title, html) = parse_and_generator(&md);
    let file_gen = format!("website/{}/{}.html", parent.to_str().unwrap(), file);
    fs::write(file_gen, html).expect("Something went wrong");

    Ok((title, format!("{}/{}", parent.to_str().unwrap(), file)))
}

fn create_directory(
    dir: &Path,
    cb: &dyn Fn(&DirEntry) -> io::Result<(Text, Link)>,
) -> io::Result<(Vec<String>, HashMap<Link, Text>)> {
    let mut sorted_by_modification: Vec<(String, SystemTime)> = Vec::new();
    let mut pages = HashMap::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
               create_directory(&path, cb)?;
            } else {
                let (title, link) = cb(&entry).unwrap();
                pages.insert(link.clone(), title);

                let metadata = fs::metadata(&path)?;
                if let Ok(modified_time) = metadata.modified() {
                    sorted_by_modification.push((link, modified_time));
                } else {
                    panic!("Cannot get modified time");
                }
            }
        }
    }
    sorted_by_modification.sort_by(|a, b| b.1.cmp(&a.1));
    let files = sorted_by_modification
        .iter()
        .map(|x| x.0.to_string())
        .collect();

    Ok((files, pages))
}

fn create_index(files: Vec<String>, pages: HashMap<Link, Text>) -> io::Result<()> {
    // make a configuration file
    let mut html = String::from(
        "<!DOCTYPE html>
<html lang=\"en\">

<head>
  <title>ParaPsychic's Blog</title>
  <meta charset=\"UTF-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <link href=\"css/style.css\" rel=\"stylesheet\">
</head>
<body>
    <H1> ParaPsychic's Blog </H1></br>
",
    );

    for entry in files {
        html.push_str(&format!("<p><a href={}.html>{}</a></p>", entry, pages[&entry]))
    }

    html.push_str("</body>\n</html>");

    fs::write("website/index.html", html).unwrap();

    Ok(())
}
