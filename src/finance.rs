use crate::{llm::LLM, llm::Model, GenericError};
use std::{io::{self, Write}, path::Path, thread::sleep, time::Duration};
use std::fs::File;
use std::io::Read;

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
        - He Never mentions that he's giving an answer for eg heres a summary etc.
        - Mustafa tries to phrase his answers as a concise report.
        - He does not address anyone in his answer, instead writes his answers as a report.
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

        And proceed forwards by pressing ENTER.

        "#);

        io::stdout().flush().unwrap(); // Flush the stdout buffer to ensure the prompt is printed
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        println!("Enter the path to your financials: ");

        io::stdout().flush().unwrap(); // Flush the stdout buffer to ensure the prompt is printed
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        let statement_file = input.trim();

        self.aggregate_data(statement_file)?;

        Ok(())
    }

    fn read_income_statements(&mut self, mut file: String) -> Result<String, GenericError>{

        file.push_str("/income_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        I want you analyze the provided income statement in detail, quoting your figures for the stock ticker {}
        The income statement is as follows: {}
        Make a one page executive summary of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }
    
    fn read_cash_flow_statement(&self, mut file: String) -> Result<String, GenericError>{
        file.push_str("/cash_flow_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        I want you analyze the provided cashflow statement in detail, quoting your figures for the stock ticker {}
        The cashflow statement is as follows: {}
        Make a one page executive summary of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }

    fn read_balance_sheet(&self, mut file: String) -> Result<String, GenericError> {
        file.push_str("/balance_sheet_statement.txt");

        let file = std::fs::read_to_string(file)?;

        let prompt = format!(r#"
        I want you analyze the provided balance sheet statement in detail, quoting your figures for the stock ticker {}
        The balance sheet statement is as follows: {}
        Make a one page executive summary of your findings.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }

    fn read_reports(&self, mut path: String) -> Result<Vec<String>, GenericError>{
        use poppler::Document;

        let mut summarized_file = path.clone();

        let mut content = Vec::new();

        let mut summaries: Vec<String> = Vec::new();

        path.push_str("/reports/2023.pdf");

        let path : &Path = Path::new(&path);
    
        File::open(path)
            .and_then(|mut file| file.read_to_end(&mut content))
            .map_err(|_| {
                eprintln!("ERROR: could not read file");
            }).unwrap();
    
        let pdf = Document::from_data(&content, None).map_err(|_| {
            eprintln!("ERROR: could not read file")
        }).unwrap();
    
        let n = pdf.n_pages();
        for i in 0..n {
            let page = pdf.page(i).expect(&format!("{i} is within the bounds of the range of the page"));
            if let Some(content) = page.text() {
                let prompt = format!(r#"
                You are being given information page by page.
                For each page, give a summary as short as possible to conserve token count and space.
                If you feel the information is redundant,feel free to ignore. I want the information summarized in
                less than 100 words.
                I want to scan mainly for company related risks, issues and boons.
                If the page is not either of these things, do not return any response.
                page number {} => {}
                "#, i, content.to_string());
                let model = self.llm.clone().model.unwrap();
                let output = self.llm.prompt(Some(prompt.trim().to_string()), model, false)?;
                summaries.push(output);
            }
            println!("");
            sleep(Duration::from_secs(10));
        }


        summarized_file.push_str("/summarized_files.txt");

        let mut file = File::create(summarized_file)?;

        for line in &summaries{
            writeln!(file, "{}", line)?;
        }

        Ok(summaries)
    }

    fn aggregate_data(&mut self, statement_file : &str) -> Result<(), GenericError>{
        let income_analysis = self.read_income_statements(statement_file.to_string())?;
        sleep(Duration::from_secs(30));
        let cash_flow_ananlysis = self.read_cash_flow_statement(statement_file.to_string())?;
        sleep(Duration::from_secs(30));
        let balance_sheet_analyis = self.read_balance_sheet(statement_file.to_string())?;
        sleep(Duration::from_secs(30));
        let annual_statement = self.read_reports(statement_file.to_string())?;

        let prompt = format!(r#"
        I want you this analysis for the income statement {}
        the cash flow statement {}
        and the balance_sheet {}
        to now generate a comprehensive summary on the company, and present a use case
        for investing into the company.

        Annual statements {:?}
        "#, income_analysis, cash_flow_ananlysis, balance_sheet_analyis, annual_statement);

        
        let final_result = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA8b, true)?;

        Ok(())
    }

}