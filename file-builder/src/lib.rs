use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use parser_generator::{parse_and_generator, utils::remove_all_special};

pub fn start(folder_path: &str, website_name: &str) -> Result<(), String> {
    let start_path = Path::new(folder_path);
    create_directory(start_path)?;
    create_index(start_path, website_name)
}

fn create_directory(start_path: &Path) -> Result<(), String> {
    if let Err(_) = fs::create_dir_all("website") {
        return Err("Cannot create output directory".to_owned());
    }

    if !start_path.is_dir() {
        return Err(format!(
            "Cannot access directory: {}",
            start_path.to_str().unwrap()
        ));
    }

    let mut start_path_string = start_path.to_str().unwrap().to_string();
    let start_path_len = if start_path_string.ends_with("/") {
        start_path_string.len()
    } else {
        start_path_string.push('/');
        start_path_string.len()
    };

    let index_path: String = format!("{}index.md", start_path_string);

    let mut dir_queue: Vec<String> = vec![start_path_string];

    while !dir_queue.is_empty() {
        let path = dir_queue.pop().unwrap();
        if let Ok(entries) = fs::read_dir(path) {
            let mut dir_entries: Vec<fs::DirEntry> = entries
                .filter_map(|entry| entry.ok()) // Filter out any potential errors
                .collect();

            // Sort the entries by date modified in ascending order
            //  this is reversed when adding back
            dir_entries.sort_by(|a, b| {
                let time_a = a.metadata().unwrap().modified().unwrap();
                let time_b = b.metadata().unwrap().modified().unwrap();
                time_a.cmp(&time_b) // Compare in ascending order
            });

            for entry in dir_entries {
                let dir_str = entry.path().to_str().unwrap().to_string();
                let new_path =
                    format!("website/{}", remove_all_special(&dir_str[start_path_len..]));

                if entry.path().is_dir() {
                    if let Err(x) = fs::create_dir_all(new_path) {
                        return Err(x.to_string());
                    }

                    dir_queue.push(dir_str.to_owned());
                } else {
                    if dir_str == index_path {
                        println!("An index.md was found. 
                                     \nThis file will be treated as html and the contents would be injected into index.html without any parsing.
                                     \nThe injected html will be before the list of posts.");
                        continue;
                    }

                    handle_file(&dir_str, &new_path);
                }
            }
        }
    }

    Ok(())
}

fn handle_file(file: &str, new_path: &str) {
    let mut path_as_pieces = file.split('.').collect::<Vec<&str>>();

    match path_as_pieces.pop() {
        Some("md") => {}
        _ => {
            if let Err(_) = fs::copy(file, new_path) {
                eprintln!("Could not copy {} to the {}.", file, new_path);
            }
            return;
        }
    };

    let markdown = match fs::read_to_string(file) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Cannot read the contents of {}", file);
            return;
        }
    };

    let (title, html) = parse_and_generator(&markdown);
    let mut temp_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("website/tmp_index")
        .unwrap();

    let path = format!("{}.html", new_path.strip_suffix("md").unwrap());

    if let Err(_) = temp_file.write_fmt(format_args!(
        "<li><a href='{}'>{}</a></li>\n",
        &path["website/".len()..],
        title
    )) {
        eprintln!(
            "Could not write to {}. index.html might have unexpected results",
            path
        );
    }

    if let Err(x) = fs::write(&path, html) {
        eprintln!("Could not write to the {}. {}", path, x);
    }
}

fn create_index(start_path: &Path, website_name: &str) -> Result<(), String> {
    let mut index_path_string = start_path.to_str().unwrap().to_string();
    if index_path_string.ends_with("/") {
        index_path_string.push_str("index.md");
    } else {
        index_path_string.push_str("/index.md");
    }

    let custom_html_from_index_md = match fs::read_to_string(index_path_string) {
        Ok(x) => x,
        Err(_) => {
            format!("<H1>{}</H1></br>", website_name)
        }
    };

    let mut contents = format!(
        "<!DOCTYPE html>
<html lang=\"en\">

<head>
  <title>{}</title>
  <meta charset=\"UTF-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <link href=\"css/style.css\" rel=\"stylesheet\">
</head>
<body>
{}
",
        website_name, custom_html_from_index_md
    );

    contents.push_str("<ol>");

    let posts = match fs::read_to_string("website/tmp_index") {
        Ok(x) => x,
        Err(_) => String::new(),
    };

    posts.lines().rev().for_each(|line| {
        contents.push_str(line);
        contents.push('\n')
    });

    contents.push_str("</ol>\n</body></html>");
    if let Err(x) = fs::write("website/index.html", contents) {
        return Err(format!("Could not write to index.html, {}", x));
    }

    if let Err(_) =  fs::remove_file("website/tmp_index"){
        return Err("Could not remove tmp_index".to_string());
    }

    Ok(())
}
