use crate::{helper::{ToDocument, ToString}, llm::{Model, LLM}, GenericError};
use std::{io::{self, Write}, path::Path, process::Output, thread::sleep, time::Duration};
use std::fs::File;
use std::io::Read;
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
        - Mustafa has his output structured in HTML, with human readable formatting for the output using CSS.
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
        - Make a detailed report of your findings.
        - The output should be in HTML.
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
        - Make a detailed report of your findings.
        - The output should be in HTML.
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
        - Make a detailed report of your findings.
        - The output should be in HTML.
        "#, self.ticker, file);

        let output = self.llm.prompt(Some(prompt.trim().to_string()), Model::LLMA70b, false)?;

        Ok(output)
    }

    fn read_reports_for_issues(&self, mut path: String) -> Result<String, GenericError>{
        use poppler::Document;

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
                - You are being given investment information page by page.
                - I want you to scan the issues that the company may face and record them.
                - I want you provide your analysis.
                - Keep it brief (<= 50 words)
                - I just want the spark notes, no analysis from yourself.
                - It is imperitative to follow this format:
                **PAGE NUMBER: **
                **REPORT: **
                - current page number {} => {}
                "#, i, content.to_string());
                let model = Model::LLMA70b;
                let output = self.llm.prompt(Some(prompt.trim().to_string()), model, true)?;
                summaries.push(output);
                println!("");
            }
            sleep(Duration::from_secs(30));
        }

        let summaries_string = summaries.to_string().unwrap();

        println!("{}", summaries_string);

        let prompt = format!(r#"
        - Prepare an executive report from the information being given to you.
        information: {}
        "#, summaries_string);

        let model = Model::LLMA8b;
        let output = self.llm.prompt(Some(prompt.trim().to_string()), model, false)?;

        Ok(output)

    }

    fn aggregate_data(&mut self, statement_file : &str) -> Result<(), GenericError>{

        // println!("Reading income statement ..");
        // let income_analysis = self.read_income_statements(statement_file.to_string())?;
        // income_analysis.write_to_file(&format!("{}/analysis/{}", statement_file, "income_analysis.html"))?;
        // sleep(Duration::from_secs(30));
        // println!("Reading cash flow statement ..");
        // let cash_flow_analysis = self.read_cash_flow_statement(statement_file.to_string())?;
        // cash_flow_analysis.write_to_file(&format!("{}/analysis/{}", statement_file, "cash_flow_analysis.html"))?;
        // sleep(Duration::from_secs(30));
        // println!("Reading balance sheet statement ..");
        // let balance_sheet_analyis = self.read_balance_sheet(statement_file.to_string())?;
        // balance_sheet_analyis.write_to_file(&format!("{}/analysis/{}", statement_file, "balance_sheet_analysis.html"))?;

        let output = self.read_reports_for_issues(statement_file.to_string())?;
        output.write_to_file(&format!("{}/analysis/{}", statement_file, "issues_report.html"))?;

        Ok(())
    }

}