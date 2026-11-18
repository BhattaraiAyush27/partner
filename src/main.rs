use dotenvy::dotenv;
use serde_json::{Value, json};
use std::{env, error::Error, fs, time::Duration};

struct Params {
    instruction: String,
    path: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv()?;
    let skill = include_str!("../skills/fix-regression/SKILL.md");
    let args = &Params {
        instruction: std::env::args().nth(2).expect("No filepath given"),
        path: std::path::PathBuf::from(std::env::args().nth(1).expect("No filepath given")),
    };
    let source_file = &args.path.to_string_lossy();
    let source = fs::read_to_string(&args.path)?;
    let instruction = &args.instruction;

    let response: Value = reqwest::blocking::Client::new()
        .post("https://openrouter.ai/api/v1/responses")
        .timeout(Duration::from_secs(200))
        .bearer_auth(env::var("PARTNER_API_KEY")?)
        .json(&json!({
            "model": env::var("PARTNER_MODEL")?,
            "instructions": skill,
            "input": format!(
                "{instruction} \n\
                Source file: {source_file}\n\n{source}
                "
            ),
            "store": false
        }))
        .send()?
        .error_for_status()?
        .json()?;

    let text = response["output"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| item["type"] == "message")
        .flat_map(|item| item["content"].as_array().into_iter().flatten())
        .filter(|part| part["type"] == "output_text")
        .filter_map(|part| part["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if text.trim().is_empty() {
        return Err("The model returned no text suggestion".into());
    }

    println!("{text}");
    Ok(())
}
