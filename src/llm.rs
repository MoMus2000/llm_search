use std::io::{self, Write};
use reqwest;
use std::{collections::HashMap, env};
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

use crate::GenericError;

pub struct LLM {
    pub ticker: Option<String>,
    pub system: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Serialize, Debug)]
struct Payload{
    messages: Vec<HashMap<String, String>>,
    model: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Choices {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    index: i32,
    message: Message,
    logprobs: Option<serde_json::Value>, // Assuming logprobs can be null or some JSON structure
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug)]
pub enum Model{
    LLMA8b,
    LLMA70b,
    MISTRAL,
    GEMMA7b,
    GEMMA9b
}

impl Into<String> for &Model {
    fn into(self) -> String {
        match *self {
            Model::LLMA8b => String::from("llama3-8b-8192"),
            Model::LLMA70b => String::from("llama3-70b-8192"),
            Model::MISTRAL => String::from("mixtral-8x7b-32768"),
            Model::GEMMA7b => String::from("gemma-7b-it"),
            Model::GEMMA9b => String::from("gemma-9b-it"),
        }
    }
}

impl LLM {
    pub fn new() -> LLM {
        Self{
            ticker: None,
            system: None,
            prompt: None
        }
    }

    pub fn context_prompt(&self, look_back: usize, model:Model) -> Result<(), GenericError>{
        let mut look_back_window : Vec<String> = Vec::new();
        let mut look_back_input_window : Vec<String> = Vec::new();
        let mut look_back_assistant_window : Vec<String> = Vec::new();

        let mut message = String::from("");
        loop{
            let mut input = String::new();

            print!("Please enter some input: ");
            io::stdout().flush().unwrap(); // Flush the stdout buffer to ensure the prompt is printed
            io::stdin().read_line(&mut input)
                .expect("Failed to read line");
            
            look_back_assistant_window.push(message.to_string());
            look_back_input_window.push(input.to_string());

            let prepared_prompt = self.prepare_prompt(&look_back_input_window, &look_back_assistant_window, &model);

            let groq_api_key = env::var_os("GROQ_API_KEY")
                .expect("Key Not Found")
                .into_string()
                .unwrap();

            let url = "https://api.groq.com/openai/v1/chat/completions";

            let request = reqwest::blocking::Client::new();

            let response = request.post(url)
            .header("Authorization", format!("Bearer {}", groq_api_key))
            .header("CONTENT_TYPE", "application/json")
            .json(&prepared_prompt)
            .send()?;

            let response : Choices = response.json()?;

            message = response.choices.get(0).expect("").message.content.to_string();

            let delay = Duration::from_millis(5); // Adjust the delay as needed

            for char in message.chars() {
                print!("\x1b[38;2;255;100;0m{}\x1b[0m", char); // Orange color
                io::stdout().flush().unwrap(); // Flush stdout to ensure the character is printed immediately
                sleep(delay);
            }

            println!(); // Move to the next line after the animation is done

            let count = &look_back_window.len();

            if count > &look_back{
                look_back_window.push(message.to_string());
                look_back_window.remove(0);
            }
            else{
                look_back_window.push(message.to_string());
            }

        }

    }

    fn prepare_prompt(&self, look_back_input_window: &Vec<String>, look_back_assistant_window: &Vec<String>, model: &Model) -> Payload{
        let mut prompt_assistant_vec : Vec<HashMap<String, String>> = Vec::new();
        let mut prompt_input_vec : Vec<HashMap<String, String>> = Vec::new();

        let mut result_vec : Vec<HashMap<String, String>> = Vec::new();

        for prev_response in look_back_assistant_window{
            let mut assistant_hashmap : HashMap<String, String>= HashMap::new();
            assistant_hashmap.insert("role".to_string(), "assistant".to_string());
            assistant_hashmap.insert("content".to_string(), prev_response.to_string());

            prompt_assistant_vec.push(assistant_hashmap);
        }

        for prev_response in look_back_input_window{
            let mut user_map: HashMap<String, String> = HashMap::new();

            user_map.insert("role".to_string(), "user".to_string());
            user_map.insert("content".to_string(), prev_response.to_string());

            prompt_input_vec.push(user_map);
        }


        let mut system_map: HashMap<String, String> = HashMap::new();

        if self.system.is_some(){
            system_map.insert("role".to_string(), "system".to_string());
            let system_resp = self.system.clone().unwrap().to_string();
            system_map.insert("content".to_string(), system_resp);
            result_vec.push(system_map);
        }

        for i in 0 .. prompt_assistant_vec.len(){
            result_vec.push(prompt_assistant_vec.get(i).unwrap().clone());
            result_vec.push(prompt_input_vec.get(i).unwrap().clone());
        }

        let model_str: String = (model).into();

        Payload{
            model: model_str,
            messages: result_vec
        }

    }

    pub fn prompt(&self, query : Option<String>, model: Model) -> Result<(), GenericError>{
        let groq_api_key = env::var_os("GROQ_API_KEY")
            .expect("Key Not Found")
            .into_string()
            .unwrap();

        let url = "https://api.groq.com/openai/v1/chat/completions";

        let request = reqwest::blocking::Client::new();

        let mut user_map: HashMap<String, String> = HashMap::new();

        user_map.insert("role".to_string(), "user".to_string());
        user_map.insert("content".to_string(), query.unwrap());
        
        
        let mut vec : Vec<HashMap<String, String>>= Vec::new();
        
        vec.push(user_map);

        let mut assistant_map: HashMap<String, String> = HashMap::new();
        assistant_map.insert("role".to_string(), "assistant".to_string());
        assistant_map.insert("content".to_string(), "Try to answer as concise as possible, I do not want to read large responses".to_string());
        vec.push(assistant_map);
        
        let model_str: String = (&model).into();

        let body = Payload{
            model: model_str,
            messages: vec
        };

        let response = request.post(url)
        .header("Authorization", format!("Bearer {}", groq_api_key))
        .header("CONTENT_TYPE", "application/json")
        .json(&body)
        .send()?;

        let response : Choices = response.json()?;

        println!("{}", response.choices.get(0).expect("Expected a response from the LLM").message.content);
        
        Ok(())
    }
}