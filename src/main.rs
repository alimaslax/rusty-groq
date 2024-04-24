use dialoguer::{ theme::ColorfulTheme, Select };
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use std::env::{ self };
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
    let url = "http://localhost:11434/api/chat";
    let args: Vec<String> = env::args().collect();
    let prompt = args[1..].join(" ");

    let request_body =
        json!({
        "messages": [
            {
                "role": "system",
                "content": "Valid single-line MacOSX bash script only. Assume user has needed commands. DO NOT give any expected outputs, or additional text that will make an invalid bash cli call.\n
                Use this reference for example. Q: prep one-app for me. A: cd ~/dev/one-app/apps/universal && yarn && yarn prebuild && open ios/NetJetsDEV.xcworkspace\nQ: go home and do a git status. A: cd ~/ && git status\n
                Q: go to dev one-app. A: cd ~/dev/one-app/apps/universal && cd ~/dev/one-app\nQ: run op-be4fe. A: cd ~/dev/op-be4fe/ && dotnet run\nQ: run dotnet for me. A: cd ~/dev/op-be4fe/ && dotnet run\n"
            },
            {
                "role": "user",
                "content": "fetch flydev version"
            },
            {
                "role": "assistant",
                "content": "curl -s https://flydev.netjets.com/api/diagnostics/version | jq ."
            },
            {
                "role": "user",
                "content": "go home and do a git status"
            },
            {
                "role": "assistant",
                "content": "cd ~/ && git status"
            },
            {
                "role": "user",
                "content": prompt.trim()
            },
        ],
        "model": "phi3:3.8b-mini-instruct-4k-q4_K_M"
    });

    let client = Client::new();

    // create another client object

    let res = client
        .post(url)
        //.header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send().await?;

    let res_text = res.text().await?;

    let mut deserializer = serde_json::Deserializer::from_reader(res_text.as_bytes());

    let mut responses = Vec::new();

    while let Ok(response) = Response::deserialize(&mut deserializer) {
        responses.push(response);
    }

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
