use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
};
use tokio::main;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Describe what the program should do, be as specific as possible
    #[arg(short, long)]
    description: String,

    /// The programming language to use
    #[arg(short, long, default_value = "javascript")]
    language: String,

    /// The name of the project which will also be the name of the directory created
    #[arg(short, long, default_value = "myapp")]
    name: String,

    /// The model to use, see https://platform.openai.com/docs/models for specific models
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,

    /// Max allowed tokens
    /// See https://beta.openai.com/docs/api-reference/completions/create#max_tokens
    /// for more information
    /// Default: 2048
    #[arg(short, long, default_value = "2048")]
    tokens: u16,
}

#[derive(Deserialize, Serialize)]
struct OutputJson {
    dockerfile: String,
    makefile: String,
    source_files: Vec<SourceFile>,
    readme: String,
    joke: String,
}

#[derive(Deserialize, Serialize)]
struct SourceFile {
    name: String,
    contents: String,
}

#[main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let prompt = format!(
        "Take the following programming language, application requirements, and produce a working application.

        your solution must include:
        1. Dockerfile that allows the application to be built and run
        2. Makefile that contains the following commands assuming that the application is executed using the Dockerfile.
            a. make build
            b. make run (make sure that docker cleans up after itself)
            c. make test (make sure that docker cleans up after itself)
        3. Readme with instructions required to build and run the application
        4. files with the source code for the application, make sure to not escape the control characters twice, like \\n because that will break the source code.
        5. Make sure to always include a json property called \"joke\" with a joke about software developers.


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
            \"joke\": \"joke contents\",
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
        name = args.name,
        description = args.description,
        language = args.language
    );
    println!("Sending prompt: {}", prompt);

    let client = Client::new();
    let req = CreateChatCompletionRequestArgs::default()
        .max_tokens(args.tokens)
        .model(args.model)
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(
                    "You are a helpful programming assistant.
                    You are expected to process an application description and generate the files and steps necessary to create the application using your language model.
                    You can only respond with a Json object that matches the provided output schema.
                    The returned Json can include an array of objects as defined by the output schema.
                    You are not allowed to return anything but a valid Json object.",
                )
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(prompt)
                .build()?,
        ])
        .build()?;
    println!("Sending prompt to OpenAI, please wait... 🤖");
    let res = client.chat().create(req).await?;
    println!("Got a response ✅ Attempting to decode the contents...");
    println!("Response:\n{}", &res.choices[0].message.content);
    let contents: OutputJson = serde_json::from_str(&res.choices[0].message.content)
        .map_err(|e| {
            println!(
                "Failed to decode the contents, please try again. Sometimes OpenAI returns invalid JSON."
            );
            e
        })?;
    println!("Success, the robot has obeyed our orders.\n");

    println!("Generating the project files... 🤖");

    // Create a folder with the project name
    let project_name = args.name;
    let project_path = format!("./{}", project_name);
    println!("Creating project folder `{}`", project_path);
    fs::create_dir_all(&project_path)?;

    // Create a dockerfile
    let dockerfile_path = format!("{}/Dockerfile", project_path);
    println!("Creating dockerfile `{}`", dockerfile_path);
    let dockerfile_contents = contents.dockerfile;
    fs::write(&dockerfile_path, dockerfile_contents)?;

    // Create a makefile
    let makefile_path = format!("{}/Makefile", project_path);
    println!("Creating makefile `{}`", makefile_path);
    let makefile_contents = contents.makefile;
    fs::write(&makefile_path, makefile_contents)?;

    // Create a readme
    let readme_path = format!("{}/README.md", project_path);
    println!("Creating readme `{}`", readme_path);
    let readme_contents = contents.readme;
    fs::write(&readme_path, readme_contents)?;

    // Create source files
    let source_files_path = &project_path;
    println!("Creating source files folder `{}`", source_files_path);
    // iterate through the source files and create them
    for source_file in contents.source_files {
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
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
        let source_file_contents = source_file.contents;
        fs::write(&source_file_path, source_file_contents)?;
        println!("Created source file `{}`", source_file_path);
    }

    println!("Project files generated successfully ✅\n");
    println!("{}\n", contents.joke);
    println!("Disclaimer: This project was generated by a robot, please review the code before executing it.\n");
    println!("To execute the project, run the following commands:\n");
    println!("cd {}", project_name);
    println!("make build");
    println!("make run");
    // print disclaimer about the project

    Ok(())
}
