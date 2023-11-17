use crate::misc::{RequestError, Weather, get_current_dir};
use chrono::{DateTime, Local};
use reqwest::{Url, StatusCode, Client, Response};
use std::error::Error;
use std::fs::{metadata, read_to_string};
use std::io;

pub async fn get_weather(zip_code: &str, hour_12: &bool) -> Result<Weather, Box<dyn Error>> {
    let raw_url: String = format!("http://wttr.in/{zip_code}?format=j1");
    let escaped_url: Url = match Url::parse(raw_url.as_str()){
        Ok(url) => url,
        Err(_) => {
            let request_error: Box<RequestError> = Box::<RequestError>::new(
                RequestError::new(StatusCode::NOT_FOUND),
            );
            return Err(request_error);
        }
    };
    let client: Client = Client::builder().use_rustls_tls().build().unwrap();

    let wttr_response: Response = match client.get(escaped_url).send().await {
        Ok(resp) => resp,
        Err(_) => {
            let (saved_json, weather_time, weather_date): (String, String, String) =
                match get_weather_from_saved_json() {
                    Ok(json_data) => json_data,
                    Err(_) => {
                        let request_error: Box<RequestError> = Box::<RequestError>::new(
                            RequestError::new(StatusCode::GATEWAY_TIMEOUT),
                        );
                        return Err(request_error);
                    }
                };
            return Ok(Weather::new(
                saved_json, 
                weather_time, 
                weather_date, 
                hour_12.to_owned()
            )?);
        }
    };

    match wttr_response.status() {
        StatusCode::OK => {
            let weather_json: String = wttr_response.text().await?;
            if weather_json.contains("Unknown location; please try") {
                let (saved_json, weather_time, weather_date): (String, String, String) =
                    match get_weather_from_saved_json() {
                        Ok(json_data) => json_data,
                        Err(_) => {
                            let request_error: Box<RequestError> = Box::<RequestError>::new(
                                RequestError::new(StatusCode::TOO_MANY_REQUESTS),
                            );
                            return Err(request_error);
                        }
                    };
                return Ok(Weather::new(
                    saved_json, 
                    weather_time, 
                    weather_date,
                    hour_12.to_owned()
                )?);
            } else {
                let weather_update_time: String = Local::now().format("%I:%M %P").to_string();
                let weather_update_date: String = Local::now().format("%D").to_string();
                return Ok(Weather::new(
                    weather_json,
                    weather_update_time,
                    weather_update_date,
                    hour_12.to_owned()
                )?);
            }
        }
        _ => {
            let (saved_json, weather_time, weather_date): (String, String, String) =
                match get_weather_from_saved_json() {
                    Ok(json_data) => json_data,
                    Err(_) => {
                        let request_error: Box<RequestError> =
                            Box::<RequestError>::new(RequestError::new(wttr_response.status()));
                        return Err(request_error);
                    }
                };
            return Ok(Weather::new(
                saved_json, 
                weather_time, 
                weather_date,
                hour_12.to_owned()
            )?);
        }
    };
}

fn get_weather_from_saved_json() -> Result<(String, String, String), io::Error> {

    let settings_file: std::path::PathBuf = get_current_dir();
    let file_path: std::path::PathBuf = settings_file.join("last_weather.json");
    let file_name: &str = file_path.to_str().expect("Invalid saved weather file.");

    let saved_json: String = read_to_string(file_name)?;
    let json_modified: DateTime<Local> = metadata(file_name)?.modified()?.into();
    let json_time: String = json_modified.format("%I:%M %P").to_string();
    let json_date: String = json_modified.format("%D").to_string();

    if saved_json.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No saved weather json found."));
    };

    Ok((saved_json, json_time, json_date))
}