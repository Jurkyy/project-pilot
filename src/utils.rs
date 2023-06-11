use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct OutputJson {
    pub dockerfile: String,
    pub makefile: String,
    pub readme: String,
    pub source_files: Vec<SourceFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceFile {
    pub name: String,
    pub contents: String,
}

pub fn generate_prompt(name: &str, description: &str, language: &str) -> String {
    format!(
        "Take the following programming language, application requirements, and produce a working application.

        your solution must include:
        1. Dockerfile that allows the application to be built and run
        2. Makefile that contains the following commands assuming that the application is executed using the Dockerfile.
            a. make build
            b. make run (make sure that docker cleans up after itself)
            c. make test (make sure that docker cleans up after itself)
        3. Readme with instructions required to build and run the application
        4. files with the source code for the application, make sure to not escape the control characters twice, like \\n because that will break the source code.


        The output must match the provided output json schema and be a valid json.

        Project Name:
        ---
        {name}
        ---

        Programming Language:
        ---
        {language}
        ---

        Application Requirements:
        ---
        {description}
        ---

        Output Json schema:
        {{
            \"dockerfile\": \"dockerfile contents\",
            \"makefile\": \"makefile contents\",
            \"readme\": \"readme contents\",
            \"source_files\": [
                {{
                    \"name\": \"...\",
                    \"contents\": \"...\"
                }}
            ]
        }}

        Make sure that you do not include invalid control characters in the output json or invalid characters (like double quotes or something else that can break the json).

        Respond ONLY with the data portion of a valid Json object. No schema definition required. No other words.",
        name = name,
        description = description,
        language = language
    )
}

pub fn create_file(file_path: &str, file_contents: &str) -> anyhow::Result<()> {
    println!("Creating file `{}`", file_path);
    fs::write(file_path, file_contents)?;
    Ok(())
}

pub fn create_source_files(
    source_files_path: &str,
    source_files: Vec<SourceFile>,
) -> anyhow::Result<()> {
    println!("Creating source files folder `{}`", source_files_path);

    for source_file in source_files {
        if source_file.name.to_lowercase().contains("makefile")
            || source_file.name.to_lowercase().contains("dockerfile")
            || source_file.name.to_lowercase().contains("readme")
        {
            println!(
                "Skipping source file `{}` because it was already created",
                source_file.name
            );
            continue;
        }

        let source_file_path = format!("{}/{}", source_files_path, source_file.name);
        let parent = Path::new(&source_file_path).parent().unwrap();
        fs::create_dir_all(parent)?;

        create_file(&source_file_path, &source_file.contents)?;
        println!("Created source file `{}`", source_file_path);
    }

    Ok(())
}
