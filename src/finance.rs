use crate::{helper::{ToDocument, ToString}, llm::{Model, LLM}, GenericError};
use std::{fmt::format, io::{self, Write}, path::Path, process::Output, thread::sleep, time::Duration};
use std::fs::File;
use std::io::Read;
use clap::Error;
use indicatif::{ProgressBar, ProgressStyle};


#[derive(Clone)]
pub struct Finance{
    ticker: String,
    llm : LLM
}

impl Finance{
    pub fn new(ticker: String, llm: LLM) -> Self{
        Finance{ticker, llm}
    }

    pub fn run(&mut self) -> Result<(), GenericError>{
        self.llm.system = Some(r#"
        Please keep the following in mind when I ask you questions from now on: 
        Act as a financial analyst named Mustafa. Mustafa has an MBA in Finance, and he is a CFA. He has a lot of experience in financial analysis and investing in companies. He is dedicated to helping companies and teams with their company analysis. Mustafa believes in 360 analysis with very detailed overviews of the company's leadership teams, qualitative research, quantitative research and financial research.
        Writing Style: Mustafa's writing style is very direct, concise and non-apologetic with a goal to help his clients. Mustafa follows strict set of rules:
        - He NEVER mentions that you're an AI.
        - Mustafa avoids any language constructs that could be interpreted as expressing remorse, apology, or regret. This includes any phrases containing words like 'sorry', 'apologies', 'regret', etc., even when used in a context that isn't expressing remorse, apology, or regret.
        - If events or information are beyond your scope or knowledge cutoff date in September 2021, provide a response stating 'I don't know' without elaborating on why the information is unavailable.
        - Refrain from disclaimers about you not being a professional or expert.
        - Never suggest seeking information from elsewhere.
        - Always focus on the key points in my questions to determine my intent.
        - Break down complex problems or tasks into smaller, manageable steps and explain each one using reasoning.
        - If a mistake is made in a previous response, recognize and correct it.
        - He Never mentions that he's giving an answer for eg using the phrase heres a summary, or here you go and things similar to that nature.
        - Mustafa does not assume that his clients know financial jargon, therefore he tries to explain all financial concepts when creating his report.
        - Mustafa is mindful of figures, million or billion that he mentions in his report.
        "#.trim().to_string());

        let mut input = String::new();

        print!(r#"

        Hello,

        We are going to perform some Company valuation and analysis ..
        To start please have the following handy:

        1. Income Statement
        2. Cash Flow Statement
        3. Balance Sheet

        If you have the following, please make sure that they are in
        a folder named after the company ticker,

        with the names

        income_statement.txt
        cash_flow_statement.txt
        balance_sheet.txt

        reports/ (Can contain the annual and quaterly reports)

        And proceed forwards by pressing ENTER.

        "#);

        io::stdout().flush().unwrap(); // Flush the stdout buffer to ensure the prompt is printed
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        let statement_file = "/Users/mmuhammad/Documents/financials/";

        let statement_file = format!("{}{}",statement_file,self.ticker);

        self.aggregate_data(&statement_file)?;

        Ok(())
    }

    fn read_income_statements(&mut self, mut file: String) -> Result<String, GenericError>{

        file.push_str("/income_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        - I want you analyze the provided income statement in detail for the stock ticker {}
        - I want to break information down by both annual and quarter.
        - The income statement is as follows: {}
        - Please write in paragraphs and use spaces to make things easier to read.
        - It is imperative for each heading to be on a new line.
        - Make a detailed report of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }
    
    fn read_cash_flow_statement(&self, mut file: String) -> Result<String, GenericError>{
        file.push_str("/cash_flow_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        - I want you analyze the provided cash flow statement in detail for the stock ticker {}
        - I want to break information down by both annual and quarter.
        - The cash flow statement is as follows: {}
        - Please write in paragraphs and use spaces to make things easier to read.
        - It is imperative for each heading to be on a new line.
        - It is imperative for you to respect and avoid tampering with financial figures. It is imperitive to not interchange millions and billions, and substitute a comma with a period and so on.
        - Make a detailed report of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }

    fn read_balance_sheet(&self, mut file: String) -> Result<String, GenericError> {
        file.push_str("/balance_sheet_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        - I want you analyze the provided balance sheet statement in detail for the stock ticker {}
        - I want to break information down by both annual and quarter.
        - The balance sheet statement is as follows: {}
        - Please write in paragraphs and use spaces to make things easier to read.
        - It is imperative for each heading to be on a new line.
        - Make a detailed report of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }

    fn read_report(&self, mut path: String) -> Result<String, GenericError>{
        use poppler::Document;

        let mut content = Vec::new();

        let mut summaries: Vec<String> = Vec::new();

        let path : &Path = Path::new(&path);

        File::open(path)
            .and_then(|mut file| file.read_to_end(&mut content))
            .map_err(|e| {
                eprintln!("ERROR: could not read file {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
    
        let pdf = Document::from_data(&content, None).map_err(|e| {
            eprintln!("ERROR: could not read file");
            Box::new(e) as Box<dyn std::error::Error>
        })?;
    
        let n = pdf.n_pages();
        for i in 0..n {
            let page = pdf.page(i).expect(&format!("{i} is within the bounds of the range of the page"));
            if let Some(content) = page.text() {
                let prompt = format!(r#"
                - You are being given investment information page by page.
                - I want you to scan through the information.
                - I want you explain whats being said on the page and summarize it into something easily digestable for someone that is not financially literate or savvy.
                - It is imperative for you to respect and avoid tampering with financial figures. It is imperitive to not interchange millions and billions, and substitute a comma with a period and so on.
                - It is imperitative to follow this format:
                **PAGE NUMBER: **
                **REPORT: **
                - current page number {} => {}
                "#, i, content.to_string());
                let model = Model::LLMA70b;
                let output = self.llm.prompt(Some(prompt.trim().to_string()), model, true);
                let output = match output{
                    Ok(res) => res,
                    Err(_) => {
                        println!("ERROR: Rerun prompt .. {}", i);
                        sleep(Duration::from_secs(120));
                        let model = Model::LLMA70b;
                        self.llm.prompt(Some(prompt.trim().to_string()), model, true).unwrap()
                    }
                };
                summaries.push(output);
                println!("");
            }
            sleep(Duration::from_secs(30));
            let summary_path = format!("/Users/mmuhammad/Documents/financials/{}/analysis/summaries.txt", self.ticker);
            let summaries_string = summaries.to_string()?;
            summaries_string.write_to_file(&summary_path)?;
        }

        let summaries_string = summaries.to_string()?;

        Ok(summaries_string)

    }

    fn aggregate_data(&mut self, statement_file : &str) -> Result<(), GenericError>{

        // println!("Reading income statement ..");
        // let income_analysis = self.read_income_statements(statement_file.to_string())?;
        // income_analysis.write_to_file(&format!("{}/analysis/{}", statement_file, "income_analysis.txt"))?;
        // sleep(Duration::from_secs(30));
        // println!("Reading cash flow statement ..");
        // let cash_flow_analysis = self.read_cash_flow_statement(statement_file.to_string())?;
        // cash_flow_analysis.write_to_file(&format!("{}/analysis/{}", statement_file, "cash_flow_analysis.txt"))?;
        // sleep(Duration::from_secs(30));
        // println!("Reading balance sheet statement ..");
        // let balance_sheet_analyis = self.read_balance_sheet(statement_file.to_string())?;
        // balance_sheet_analyis.write_to_file(&format!("{}/analysis/{}", statement_file, "balance_sheet_analysis.txt"))?;
        // println!("Reading Reports ..");

        let reports= std::fs::read_dir(format!("/Users/mmuhammad/Documents/financials/{}/reports", self.ticker))?;

        for report in reports{
            let report_name = format!("{}", report.unwrap().file_name().to_str().unwrap());
            let report_path = format!("/Users/mmuhammad/Documents/financials/{}/reports/{}", self.ticker, report_name);
            let output = match self.read_report(report_path){
                Ok(output) => output,
                Err(_) => {
                    continue
                }
            };
            output.write_to_file(&format!("{}/analysis/{}", statement_file, report_name))?;
        }

        Ok(())

    }

}