use dialoguer::{ theme::ColorfulTheme, Select };
use reqwest::Client;
use serde::{ Deserialize, Serialize };
use std::env;
use std::fs::read_to_string;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct Response {
    message: Message,
}
#[derive(Serialize, Deserialize, Debug)]
struct RequestBody {
    messages: Vec<Message>,
    model: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://localhost:11434/api/chat";
    let args: Vec<String> = env::args().collect();

    let mut prompt = String::new();

    let mut file_path = None;

    for arg in args.iter().skip(1) {
        if arg == "-f" {
            file_path = args.get(
                args
                    .iter()
                    .position(|r| r == arg)
                    .unwrap() + 1
            );
        } else {
            prompt.push_str(arg);
            prompt.push(' ');
        }
    }

    if let Some(file_path) = file_path {
        prompt = read_to_string(file_path)?;
        prompt = format!("[FILE] {}", prompt.trim());
    }
    let current_dir = env::current_dir().unwrap();
    let json_path = current_dir.join("src/prompts/bash-cli.json");
    let json_string = read_to_string(json_path).expect("Failed to read file");
    let mut request_body: RequestBody = serde_json::from_str(&json_string).unwrap();

    let prompt = prompt.trim().to_string();
    request_body.messages.push(Message {
        role: "user".to_string(),
        content: prompt,
    });
    let client = Client::new();

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send().await?;

    let res_text = res.text().await?;

    let mut deserializer = serde_json::Deserializer::from_reader(res_text.as_bytes());

    let mut responses = Vec::new();

    while let Ok(response) = Response::deserialize(&mut deserializer) {
        responses.push(response);
    }
    if let Some(_) = file_path {
        for response in responses {
            print!("{}", response.message.content);
        }
    } else {
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
    }

    Ok(())
}
