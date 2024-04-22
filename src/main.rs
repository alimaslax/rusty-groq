use std::env;
use std::io;
use crossterm::style::Attribute;
use crossterm::style::Color;
use crossterm::style::StyledContent;
use crossterm::style::Stylize;
use crossterm::terminal;
use serde_json::{ json, Value };
use crossterm::{
    cursor::{ self, MoveToColumn },
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers },
    execute,
    terminal::ClearType,
    style::PrintStyledContent,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("GROQ_API").expect("Please set the GROQ_API environment variable");

    let url = "https://api.groq.com/openai/v1/chat/completions";

    let mut prompt = String::new();
    println!("Enter your prompt:");
    io::stdin().read_line(&mut prompt)?;

    let request_body =
        json!({
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

    let client = reqwest::Client::new();

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

    let choices = res["choices"].as_array().unwrap();

    // Clear the terminal
    execute!(std::io::stdout(), terminal::Clear(ClearType::All))?;

    // Move the cursor to the top left
    execute!(std::io::stdout(), cursor::MoveToColumn(1))?;

    // Print the choices

    // Print the choices
    for (i, choice) in choices.iter().enumerate() {
        let content = choice["message"]["content"].as_str().unwrap();
        let lines = content.split("\n");
        for (j, line) in lines.enumerate() {
            let styled = line.with(Color::Yellow).on(Color::Blue).attribute(Attribute::Bold);
            execute!(std::io::stdout(), PrintStyledContent(styled))?;
        }
    }

    // Enable mouse capture
    execute!(std::io::stdout(), EnableMouseCapture)?;

    // Get the selected choice
    let mut selected = 0;
    loop {
        if let Event::Key(event) = event::read()? {
            match (event.code, event.modifiers) {
                (KeyCode::Up, KeyModifiers::NONE) => {
                    selected = ((selected as i32) - 1).max(0) as usize;
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    selected = ((selected as i32) + 1).min((choices.len() as i32) - 1) as usize;
                }
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    break;
                }
                _ => (),
            }
        }
        // Move the cursor to the selected choice
        execute!(std::io::stdout(), cursor::MoveToColumn(1), cursor::MoveDown(selected as u16))?;
        // Print the selected choice
        execute!(
            std::io::stdout(),
            PrintStyledContent(
                choices[selected]["message"]["content"]
                    .as_str()
                    .unwrap()
                    .with(Color::Yellow)
                    .on(Color::Blue)
                    .attribute(Attribute::Bold)
            )
        )?;
        // Disable mouse capture
        execute!(std::io::stdout(), DisableMouseCapture)?;
    }

    // Clear the terminal
    execute!(std::io::stdout(), terminal::Clear(ClearType::All))?;

    // Print the selected choice
    println!("You selected: {}", choices[selected]["message"]["content"].as_str().unwrap());

    // Reset the terminal to its default state
    execute!(std::io::stdout(), terminal::LeaveAlternateScreen)?;

    Ok(())
}
