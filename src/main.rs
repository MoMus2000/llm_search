mod llm;
mod finance;
mod helper;

use clap::{Parser, Subcommand};
use finance::Finance;
use helper::serve_files;

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
    #[clap(name = "query", about = "Send a single LLM query.")]
    Query {
        #[clap(long, help = "Singular prompt with no context")]
        prompt: String,
        #[clap(long, help = "Optional model to use for the context")]
        model: Option<String>
    },

    #[clap(name = "context", about = "Open a context to query the LLM with history kept intact.")]
    Context {
        #[clap(long, help = "Decide the lookback window that gets used to preserve context")]
        look_back: Option<i32>,
        #[clap(long, help = "Optional model to use for the context")]
        model: Option<String>,
        #[clap(long, help = "Optional system prompt to decide how the LLM should respond")]
        system: Option<String>
    },

    #[clap(name = "finance", about = "Perform a valuation for the stock in qs.")]
    Finance{
        #[clap(long, help = "Ticker symbol for the stock to value")]
        ticker: String,
        #[clap(long, help = "Optional model to use for the context")]
        model: Option<String>,
        #[clap(long, help = "Optional model to use for the context")]
        serve: bool
    },

    #[clap(name = "make_ticker", about = "Generate a folder with required files and folders.")]
    MakeTicker{
        ticker: String
    }

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
                        "L70" => llm::Model::LLMA70b,
                        "M" => llm::Model::MISTRAL,
                        "G7" => llm::Model::GEMMA7b,
                        "G9" => llm::Model::GEMMA9b,
                        _ => llm::Model::LLMA8b
                    }
                },
                None => llm::Model::LLMA8b
            };

            if prompt.len() != 0{
                llm.prompt(Some("".to_string()), model, true)?;
            }
            else{
                println!("No input provided, can't query the LLM");
            }
        }
        Some(Commands::Context { look_back , model, system}) => {
            if system.is_some(){
                llm.system = system.clone()
            }
            else{
                llm.system = Some(String::from("I want concise answers, do not give me large swath of text."))
            }
            let model = match model{
                Some(model_str) => {
                    match model_str.as_str(){
                        "L8" => llm::Model::LLMA8b,
                        "L70" => llm::Model::LLMA70b,
                        "M" => llm::Model::MISTRAL,
                        "G7" => llm::Model::GEMMA7b,
                        "G9" => llm::Model::GEMMA9b,
                        _ => llm::Model::LLMA8b
                    }
                },
                None => llm::Model::LLMA8b
            };

            if look_back.is_some() && look_back.unwrap() >= 1{
                llm.context_prompt(look_back.unwrap() as usize, model)?;
            }
            else{
                llm.context_prompt(20, model)?;
            }
        }
        Some(Commands::Finance {model, ticker, serve}) => {
            let model = match model{
                Some(model_str) => {
                    match model_str.as_str(){
                        "L8" => llm::Model::LLMA8b,
                        "L70" => llm::Model::LLMA70b,
                        "M" => llm::Model::MISTRAL,
                        "G7" => llm::Model::GEMMA7b,
                        "G9" => llm::Model::GEMMA9b,
                        _ => llm::Model::LLMA8b
                    }
                },
                None => llm::Model::LLMA8b
            };

            llm.model = Some(model);
            let mut fin = Finance::new(ticker.to_string(), llm);
            fin.run()?;

            if *serve {
                serve_files(&format!("/Users/mmuhammad/Documents/financials/{}/analysis", ticker));
            }

        }
        Some(Commands::MakeTicker {ticker}) => {
            let path = format!("/Users/mmuhammad/Documents/financials/{}",ticker);

            if !std::path::Path::new(&path).exists() {
                std::fs::create_dir(&path)?;
            }
            std::fs::File::create(format!("{}/income_statement.txt",path))?;
            std::fs::File::create(format!("{}/balance_sheet_statement.txt",path))?;
            std::fs::File::create(format!("{}/cash_flow_statement.txt",path))?;

            let mut reports = path.clone();

            reports.push_str("/reports");

            if !std::path::Path::new(&reports).exists() {
                std::fs::create_dir(&reports)?;
            }

            let mut analysis = path.clone();

            analysis.push_str("/analysis");

            if !std::path::Path::new(&analysis).exists() {
                std::fs::create_dir(&analysis)?;
            }

        }

        None => {}
    }


    Ok(())
}