use std::fs::File;
use std::io::Write;
use crate::GenericError;

pub trait ToDocument {
    fn write_to_file(&self, file_name: &str) -> Result<(), GenericError>;
}

impl ToDocument for Vec<String> {
    fn write_to_file(&self, file_name: &str) -> Result<(), GenericError> {
        let mut file = File::create(file_name)?;

        for line in self{
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }
}

impl ToDocument for String{
    fn write_to_file(&self, file_name: &str) -> Result<(), GenericError> {
        // Create or open a file called "output.txt"
        let mut file = File::create(file_name)?;
    
        // Write the string to the file
        file.write_all(self.as_bytes())?;

       Ok(()) 
    }
}

pub fn serve_files(file_dir: &str) {
    use std::process::Command;

    // Start Python HTTP server
    let mut output = Command::new("python3")
        .arg("-m")
        .arg("http.server")
        .arg("--directory")
        .arg(file_dir)
        .spawn()
        .expect("Failed to start Python HTTP server");

    println!("Python HTTP server started.");

    // Open the browser after a delay to allow the server to start
    std::thread::sleep(std::time::Duration::from_secs(1)); // Adjust delay as needed

    // Open browser using system command
    if cfg!(target_os = "linux") {
        Command::new("xdg-open")
            .arg(format!("http://localhost:8000"))
            .output()
            .expect("Failed to open browser.");
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(format!("http://localhost:8000"))
            .output()
            .expect("Failed to open browser.");
    } else {
        println!("Unsupported OS for automatic browser opening.");
    }

    // Wait for the HTTP server process to finish (not necessary if server runs indefinitely)
    let _ = output.wait();
}