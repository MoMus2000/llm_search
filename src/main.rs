mod llm;
use clap::{Parser, Subcommand};

type GenericError = Box<dyn std::error::Error>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

     /// Turn debugging information on
     #[arg(short, long, action = clap::ArgAction::Count)]
     debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand)]
enum Commands {
    #[clap(name = "query", about = "Send a single LLM query")]
    Query {
        #[clap(long, help = "Singular prompt with no context")]
        prompt: String,
        #[clap(long, help = "Optional model to use for the context")]
        model: Option<String>
    },

    #[clap(name = "context", about = "Open a context to query the LLM with history kept intact")]
    Context {
        #[clap(long, help = "Decide the lookback window that gets used to preserve context")]
        look_back: Option<i32>,
        #[clap(long, help = "Optional model to use for the context")]
        model: Option<String>,
        #[clap(long, help = "Optional system prompt to decide how the LLM should respond")]
        system: Option<String>
    },

}


fn main() -> Result<(), GenericError>{
    let cli = Cli::parse();
    let mut llm = llm::LLM::new();

    match &cli.command {
        Some(Commands::Query {prompt, model}) => {
            let model = match model{
                Some(model_str) => {
                    match model_str.as_str(){
                        "L8" => llm::Model::LLMA8b,
                        "Lb" => llm::Model::LLMA70b,
                        "M" => llm::Model::MISTRAL,
                        "G7" => llm::Model::GEMMA7b,
                        "G9" => llm::Model::GEMMA9b,
                        _ => llm::Model::LLMA8b
                    }
                },
                None => llm::Model::LLMA8b
            };

            if prompt.len() != 0{
                llm.prompt(Some("".to_string()), model)?;
            }
            else{
                println!("No input provided, can't query the LLM");
            }
        }
        Some(Commands::Context { look_back , model, system}) => {
            if system.is_some(){
                llm.system = system.clone()
            }
            let model = match model{
                Some(model_str) => {
                    match model_str.as_str(){
                        "L8" => llm::Model::LLMA8b,
                        "Lb" => llm::Model::LLMA70b,
                        "M" => llm::Model::MISTRAL,
                        "G7" => llm::Model::GEMMA7b,
                        "G9" => llm::Model::GEMMA9b,
                        _ => llm::Model::LLMA8b
                    }
                },
                None => llm::Model::LLMA8b
            };

            println!("Model set to {:?}", model);

            if look_back.is_some() && look_back.unwrap() >= 1{
                println!("Enabling context with lookback set to {}", look_back.unwrap());
                llm.context_prompt(look_back.unwrap() as usize, model)?;
            }
            else{
                println!("Opening context lookback set to 20");
                llm.context_prompt(20, model)?;
            }
        }
        None => {}
    }


    Ok(())
}