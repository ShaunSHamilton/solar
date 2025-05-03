use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use queries::{execute_query, get_measurements};
use serde_json::json;
use std::{error::Error, fmt::Write, fs};

mod queries;

const DATABASE: &str = "solar_assistant";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Get a list of all measurements
    let measurements = get_measurements()?;
    println!("Found measurements: {:?}", measurements);

    // Create an output directory if it doesn't exist
    fs::create_dir_all("data_json")?;

    for measurement in &measurements {
        let mut offset = 0;

        let escaped_measurement = measurement.replace("\"", "\\\"");

        let count_query = format!("SELECT COUNT(*) FROM \"{}\"", escaped_measurement);
        let count_output = execute_query(&count_query);
        let count_json: Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str(&count_output);

        // println!("{:?}", count_json);
        let max_rows = match count_json {
            Ok(json_data) => {
                if let Some(counts) = json_data["results"][0]["series"][0]["values"][0].as_array() {
                    counts
                        .iter()
                        .max_by(|a, b| a.as_u64().cmp(&b.as_u64()))
                        .expect(&format!(
                            "{} does not have a max count.\n{:?}",
                            measurement, json_data
                        ))
                        .as_u64()
                        .unwrap()
                } else {
                    eprintln!("Error: Unable to parse count from JSON.");
                    continue;
                }
            }
            Err(e) => {
                eprintln!(
                    "Error parsing JSON for measurement '{}': {}",
                    measurement, e
                );
                continue;
            }
        };

        let mut all_data: Vec<serde_json::Value> = Vec::with_capacity(max_rows as usize);

        let pb = ProgressBar::new(max_rows);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

        // continue;
        loop {
            pb.set_position(all_data.len() as u64);
            if all_data.len() >= max_rows as usize {
                break;
            }
            // count total number of rows
            let query = format!(
                "SELECT * FROM \"{}\" OFFSET {}",
                escaped_measurement, offset
            );

            let out = execute_query(&query);
            // Attempt to parse the JSON output
            let json_result: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(&out);

            match json_result {
                Ok(json_data) => {
                    // Extract the series data
                    // println!("'{}': {:#?}", measurement, json_data);
                    let series = json_data["results"][0]["series"].as_array();

                    if let Some(series_array) = series {
                        for s in series_array {
                            if let (Some(columns), Some(values)) =
                                (s["columns"].as_array(), s["values"].as_array())
                            {
                                for row in values {
                                    let mut data_point = serde_json::Map::new();
                                    for (i, column) in columns.iter().enumerate() {
                                        if let Some(col_name) = column.as_str() {
                                            data_point.insert(col_name.to_string(), row[i].clone());
                                        }
                                    }
                                    all_data.push(json!(data_point));
                                }
                            }
                        }
                    }

                    offset = all_data.len();
                }
                Err(e) => {
                    eprintln!(
                        "Error parsing JSON for measurement '{}': {}",
                        measurement, e
                    );
                    break;
                }
            }
        }
        // 3. Output to a JSON file
        let filename = format!("data_json/{}.json", measurement.replace(" ", "_"));
        let json_output = serde_json::to_string_pretty(&all_data)?;
        fs::write(&filename, json_output)?;

        pb.finish_with_message(format!("'{}' written to '{}'.", measurement, filename));
    }

    Ok(())
}
