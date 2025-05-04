use std::{
    error::Error,
    process::{Command, Stdio},
};

use crate::DATABASE;

pub fn get_measurements() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("influx")
        .arg("-database")
        .arg(DATABASE)
        .arg("-execute")
        .arg("SHOW MEASUREMENTS")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let error_message = format!(
            "Error executing 'SHOW MEASUREMENTS': {}",
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!("{}", error_message);
        return Err(error_message.into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let measurements: Vec<String> = stdout
        .lines()
        .skip(2) // Skip header lines
        .filter(|line| !line.trim().is_empty() && *line != "----")
        .map(|line| line.trim().to_string())
        .collect();

    Ok(measurements)
}

pub fn execute_query(query: &str) -> String {
    let output = Command::new("influx")
        .arg("-database")
        .arg(DATABASE)
        .arg("-format")
        .arg("json")
        .arg("-execute")
        .arg(query)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        panic!(
            "Error executing query: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}
