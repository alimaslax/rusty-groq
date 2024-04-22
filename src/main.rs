use std::{env, io};
use dialoguer::{theme::ColorfulTheme, Select};
use serde_json::{json, Value};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GROQ_API").expect("Please set the GROQ_API environment variable");

    let url = "https://api.groq.com/openai/v1/chat/completions";

    let mut prompt = String::new();
    println!("Enter your prompt:");
    io::stdin().read_line(&mut prompt)?;

    let request_body = json!({
        "messages": [
            {
                "role": "system",
                "content": "Give ONLY valid bash script commands for any of the user's questions and requests. Assume the user has any needed libraries installed. DO NOT give any outputs, or additional text that will make it invalid bash cli call. Always give multiple options seperated by newline '\n'"
            },
            {
                "role": "user",
                "content": "print dr"
            },
            {
                "role": "system",
                "content": "pwd"
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
        "model": "llama3-70b-8192"
    });

    let client = Client::new();

    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request_body)?)
        .send().await?;

    let res_text = res.text().await?;
    println!("{}", res_text);
    let res: Value = serde_json::from_str(&res_text)?;

    println!("JSON Response: {:#?}", res);

    let choices: &Vec<Value> = res["choices"].as_array().unwrap();

    let choices_strings: Vec<_> = choices
        .iter()
        .map(|choice| {
            let content = choice["message"]["content"].as_str().unwrap();
            content.split("\n").collect::<Vec<_>>()
        })
        .flatten()
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a choice")
        .default(0) // This will default to the first choice
        .items(&choices_strings)
        .interact_opt()
        .unwrap();

    match selection {
        Some(index) => println!("You selected: {}", choices_strings[index]),
        None => println!("You didn't select anything!"),
    }

    Ok(())
}