type GenericError = Box<dyn std::error::Error>;
mod llm;

use clap::{Parser, Subcommand};

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
    /// Send a single LLM query
    Query {
        prompt: String
    },

    // Open a context to query the LLM with history kept intact
    Context {
        look_back: Option<i32>
    }

}


fn main() -> Result<(), GenericError>{
    let cli = Cli::parse();
    let llm = llm::LLM::new();

    match &cli.command {
        Some(Commands::Query {prompt}) => {
            if prompt.len() != 0{
                llm.prompt(Some("".to_string()))?;
            }
            else{
                println!("No input provided, can't query the LLM");
            }
        }
        Some(Commands::Context { look_back }) => {
            if look_back.is_some() && look_back.unwrap() >= 1{
                println!("Enabling context with lookback set to {}", look_back.unwrap());
                llm.context_prompt(look_back.unwrap() as usize);
            }
            else{
                println!("Opening context without lookback");
                llm.context_prompt(0);
            }
        }
        None => {}
    }


    Ok(())
}