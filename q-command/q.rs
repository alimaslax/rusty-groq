use dialoguer::{theme::ColorfulTheme, Select};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use std::env::{self};
use std::process::Command;

#[derive(Deserialize, Debug)]
struct Response {
    // model: String,
    // created_at: String,
    message: Message,
    // done: bool,
    // total_duration: Option<u64>,
    // load_duration: Option<u64>,
    // prompt_eval_duration: Option<u64>,
    // eval_count: Option<u64>,
    // eval_duration: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Message {
    // role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let api_key = env::var("GROQ_API").expect("Please set the GROQ_API environment variable");

    let url = "http://localhost:11434/api/chat";
    let args: Vec<String> = env::args().collect();
    let prompt = args[1..].join(" ");

    let request_body = json!({
        "messages": [
            {
                "role": "system",
                "content": "Valid MacOSX bash scripts only. Assume user has needed commands. DO NOT give any expected outputs, or additional text that will make it invalid cli call"
            },
            {
                "role": "user",
                "content": "go to dev one-app"
            },
            {
                "role": "system",
                "content": "cd ~/dev/one-app"
            },
            {
                "role": "user",
                "content": "go home and do a git status"
            },
            {
                "role": "system",
                "content": "cd ~/ && git status"
            },
            {
                "role": "user",
                "content": prompt.trim()
            },
        ],
        "model": "llama3:8b"
    });

    let client = Client::new();

    let res = client
        .post(url)
        //.header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send()
        .await?;

    let res_text = res.text().await?;

    let mut deserializer = serde_json::Deserializer::from_reader(res_text.as_bytes());

    let mut responses = Vec::new();

    while let Ok(response) = Response::deserialize(&mut deserializer) {
        responses.push(response);
    }

    println!("JSON Responses: {:#?}", responses);

    let choices_strings: Vec<_> = responses
        .iter()
        .map(|response| response.message.content.to_string())
        .filter(|s| !s.is_empty()) // remove empty strings
        .collect::<Vec<_>>();

    let concatenated_string = choices_strings.join("");

    let choices: Vec<_> = if concatenated_string.contains('\n') {
        concatenated_string.split('\n').collect()
    } else {
        vec![&concatenated_string]
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a choice")
        .default(0) // This will default to the first choice
        .items(&choices)
        .interact_opt()
        .unwrap();

    match selection {
        Some(index) => {
            println!("You selected: {}", choices[index]);
            Command::new("sh")
                .arg("-c")
                // executed selected
                .arg(choices[index])
                .status()
                .unwrap();
        }
        None => println!("You didn't select anything!"),
    }

    Ok(())
}
