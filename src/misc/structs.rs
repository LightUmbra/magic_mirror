use chrono::{NaiveTime, ParseError, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, remove_file, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::fmt;

use crate::misc::{SettingsError, get_current_dir};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct UserSettings {
    pub zip_code: String,
    pub unit: String,
    #[serde(alias = "12_hour")]
    pub hour_12: bool,
}

impl UserSettings {
    pub fn new(settings_file: &str) -> Result<UserSettings, SettingsError>{
        if !Path::new(settings_file).exists() {
            let temp_settings = UserSettings {
                zip_code: "".to_string(),
                unit: "F".to_string(),
                hour_12: true,
            };

            let settings_json = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(settings_file){
                Ok(x) => x,
                Err(new_json_err) => return Err(SettingsError::IOError { error_desc: new_json_err.to_string() })
            };

            match serde_json::to_writer_pretty(&settings_json, &temp_settings) {
                Ok(_) => (),
                Err(new_json_err) => return Err(SettingsError::IOError { error_desc: new_json_err.to_string() })
            };
        }
        
        let settings_file: String = match fs::read_to_string(settings_file) {
            Ok(content) => content,
            Err(error_type) => {
                return Err(SettingsError::IOError { error_desc: error_type.to_string() });
            }
        };

        let user_settings: UserSettings = match serde_json::from_str(settings_file.as_str()){
            Ok(x) => x,
            Err(error_type) => {
                return Err(SettingsError::SerdeError { error_desc: error_type.to_string() });
            }
        };

        if user_settings.unit.to_lowercase() != "f" && user_settings.unit.to_lowercase() != "c" {
            return Err(SettingsError::SerdeError { error_desc: "Invalid unit.".to_string() });
        }

        Ok(user_settings)
    }
}

#[derive(Debug, Clone)]
pub struct Weather {
    pub last_time_updated: String,
    pub last_date_updated: String,
    pub daily_forecast: DailyForecast,
    pub current_weather: CurrentWeather,
}

impl Weather {
    pub fn new(weather_json: String, weather_time: String, weather_date: String, hour_12: bool) -> Result<Weather, Box<dyn Error>> {
        let mut temp_current: CurrentWeather = serde_json::from_str(weather_json.as_str())?;
        let mut temp_daily: DailyForecast = serde_json::from_str(weather_json.as_str())?;
        let sunset: String = temp_daily.weather[0].astronomy[0].sunset.clone();

        temp_current.current_condition[0].weather_desc = get_weather_desc(
            &temp_current.current_condition[0].weather_code,
            &weather_time,
            &sunset,
        );
        temp_current.current_condition[0].weather_image = get_weather_image(
            &temp_current.current_condition[0].weather_code,
            &weather_time,
            &sunset,
            false
        );
        
        let mut weather_days: Vec<ForecastDay> = Vec::new();
        for mut day in temp_daily.weather {
            day.date = reformat_date(day.raw_date.clone())?;
            day.weather_code = get_daily_weather_code(day.hourly.clone());
            day.weather_desc = get_weather_desc(
                &day.weather_code,
                &weather_time,
                &day.astronomy[0].sunset
            );
            day.weather_image = get_weather_image(
                &day.weather_code,
                &weather_time,
                &day.astronomy[0].sunset,
                true
            );

            let mut weather_hour: Vec<ForecastHour> = Vec::new();
            for mut hour in day.hourly {
                let (temp_time, hour_time): (String, NaiveTime) = reformat_time(hour.raw_time.clone(), hour_12);
                hour.time = temp_time;

                hour.weather_desc = get_weather_desc(
                    &hour.weather_code,
                    &hour_time.format("%I:%M %P").to_string(),
                    &day.astronomy[0].sunset.clone()
                );
                hour.weather_image = get_weather_image(
                    &hour.weather_code,
                    &hour_time.format("%I:%M %P").to_string(),
                    &day.astronomy[0].sunset,
                    false
                );
                
                weather_hour.push(hour.clone());
            };
            day.hourly = weather_hour;

            weather_days.push(day)
        };

        temp_daily.weather = weather_days;

        let weather_forecast: Weather = Weather {
            last_time_updated: weather_time,
            last_date_updated: weather_date,
            current_weather: temp_current,
            daily_forecast: temp_daily,
        };

        save_weather_json(weather_json)?;

        Ok(weather_forecast)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyForecast {
    pub weather: Vec<ForecastDay>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForecastDay {
    #[serde(alias = "date")]
    raw_date: String,
    #[serde(skip_deserializing)]
    pub date: String,
    #[serde(alias = "avgtempF")]
    pub avg_temp_f: String,
    #[serde(alias = "maxtempF")]
    pub max_temp_f: String,
    #[serde(alias = "mintempF")]
    pub min_temp_f: String,
    #[serde(alias = "avgtempC")]
    pub avg_temp_c: String,
    #[serde(alias = "maxtempC")]
    pub max_temp_c: String,
    #[serde(alias = "mintempC")]
    pub min_temp_c: String,
    #[serde(alias = "uvIndex")]
    pub uv_index: String,
    #[serde(skip_deserializing)]
    pub weather_code: String,
    #[serde(skip_deserializing)]
    pub weather_desc: String,
    #[serde(skip_deserializing)]
    pub weather_image: String,
    pub astronomy: Vec<Astronomy>,
    pub hourly: Vec<ForecastHour>,
}

impl fmt::Display for ForecastDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForecastDay({})", self.date)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Astronomy {
    pub moon_phase: String,
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForecastHour {
    #[serde(alias = "time")]
    raw_time: String,
    #[serde(skip_deserializing)]
    pub time: String,
    #[serde(alias = "tempF")]
    pub temp_f: String,
    #[serde(alias = "tempC")]
    pub temp_c: String,
    #[serde(alias = "FeelsLikeF")]
    pub feels_like_f: String,
    #[serde(alias = "FeelsLikeC")]
    pub feels_like_c: String,
    #[serde(alias = "chanceofrain")]
    pub chance_of_rain: String,
    #[serde(alias = "chanceofsnow")]
    pub chance_of_snow: String,
    #[serde(alias = "weatherCode")]
    pub weather_code: String,
    #[serde(skip_deserializing)]
    pub weather_desc: String,
    #[serde(skip_deserializing)]
    pub weather_image: String,
}

impl fmt::Display for ForecastHour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForecastHour({})", self.time)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentWeather {
    pub current_condition: Vec<CurrentConditions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentConditions {
    #[serde(alias = "weatherCode")]
    pub weather_code: String,
    #[serde(alias = "FeelsLikeF")]
    pub feels_like_f: String,
    #[serde(alias = "FeelsLikeC")]
    pub feels_like_c: String,
    #[serde(alias = "temp_C")]
    pub temp_c: String,
    #[serde(alias = "temp_F")]
    pub temp_f: String,
    pub humidity: String,
    pub pressure: String,
    #[serde(alias = "uvIndex")]
    pub uv_index: String,
    pub visibility: String,
    #[serde(skip_deserializing)]
    pub weather_desc: String,
    #[serde(skip_deserializing)]
    pub weather_image: String,
}

pub fn get_weather_desc(weather_code: &str, last_time_updated: &str, sunset: &str) -> String {
    let weather_desc: &str = match weather_code {
        "113" => {
            match is_night(last_time_updated, sunset, false) {
                false => "Sunny/Clear",
                true => "Clear"
            }
        },
        "116" => "Partly Cloudy",
        "119" => "Cloudy",
        "122" => "Overcast",
        "143" => "Mist",
        "176" => "Patchy rain nearby",
        "179" => "Patchy snow nearby",
        "182" => "Patchy sleet nearby",
        "185" => "Patchy freezing drizzle nearby",
        "200" => "Thundery outbreaks nearby",
        "227" => "Blowing snow",
        "230" => "Blizzard",
        "248" => "Fog",
        "260" => "Freezing fog",
        "263" => "Patchy light drizzle",
        "266" => "Light drizzle",
        "281" => "Freezing drizzle",
        "284" => "Heavy freezing drizzle",
        "293" => "Patchy light rain",
        "296" => "Light rain",
        "299" => "Moderate rain at times",
        "302" => "Moderate rain",
        "305" => "Heavy rain at times",
        "308" => "Heavy rain",
        "311" => "Light freezing rain",
        "314" => "Moderate or Heavy freezing rain",
        "317" => "Light sleet",
        "320" => "Moderate or heavy sleet",
        "323" => "Patchy light snow",
        "326" => "Light snow",
        "329" => "Patchy moderate snow",
        "332" => "Moderate snow",
        "335" => "Patchy heavy snow",
        "338" => "Heavy snow",
        "350" => "Hail",
        "353" => "Light rain shower",
        "356" => "Moderate or heavy rain shower",
        "359" => "Torrential rain shower",
        "362" => "Light sleet showers",
        "365" => "Moderate or heavy sleet showers",
        "368" => "Light snow showers",
        "371" => "Moderate or heavy snow showers",
        "374" => "Light showers of hail",
        "377" => "Moderate or heavy showers of hail",
        "386" => "Patchy light rain in area with thunder",
        "389" => "Moderate or heavy rain in area with thunder",
        "392" => "Patchy light snow in area with thunder",
        "395" => "Moderate or heavy snow in area with thunder",
        _ => "invalid"
    };

    if weather_desc == "invalid" {
        return format!("Invalid weather code: {}", weather_code);
    }
    
    weather_desc.to_string()
}

pub fn get_weather_image(weather_code: &str, last_time_updated: &str, sunset: &str, force_day: bool) -> String {
    let weather_image: &str = match is_night(last_time_updated, sunset, force_day) {
        false => match weather_code {
            "113" => "svg/wi-day-sunny.svg",
            "116" => "svg/wi-day-cloudy.svg",
            "119" => "svg/wi-day-cloudy.svg",
            "122" => "svg/wi-day-sunny-overcast.svg",
            "143" => "svg/wi-day-haze.svg",
            "176" => "svg/wi-day-sprinkle.svg",
            "179" => "svg/wi-day-snow.svg",
            "182" => "svg/wi-day-sleet.svg",
            "185" => "svg/wi-day-rain-mix.svg",
            "200" => "svg/wi-day-rain-mix.svg",
            "227" => "svg/wi-day-snow-wind.svg",
            "230" => "svg/wi-day-snow-thunderstorm.svg",
            "248" => "svg/wi-day-fog.svg",
            "260" => "svg/wi-day-fog.svg",
            "263" => "svg/wi-day-sprinkle.svg",
            "266" => "svg/wi-day-sprinkle.svg",
            "281" => "svg/wi-day-rain-mix.svg",
            "284" => "svg/wi-day-rain-mix.svg",
            "293" => "svg/wi-day-sprinkle.svg",
            "296" => "svg/wi-day-rain.svg",
            "299" => "svg/wi-day-rain.svg",
            "302" => "svg/wi-day-rain.svg",
            "305" => "svg/wi-day-rain.svg",
            "308" => "svg/wi-day-rain.svg",
            "311" => "svg/wi-day-rain-mix.svg",
            "314" => "svg/wi-day-rain-mix.svg",
            "317" => "svg/wi-day-sleet.svg",
            "320" => "svg/wi-day-sleet.svg",
            "323" => "svg/wi-day-snow.svg",
            "326" => "svg/wi-day-snow.svg",
            "329" => "svg/wi-day-snow.svg",
            "332" => "svg/wi-day-snow.svg",
            "335" => "svg/wi-day-snow-wind.svg",
            "338" => "svg/wi-day-snow-wind.svg",
            "350" => "svg/wi-day-hail.svg",
            "353" => "svg/wi-day-rain.svg",
            "356" => "svg/wi-day-rain.svg",
            "359" => "svg/wi-day-thunderstorm.svg",
            "362" => "svg/wi-day-sleet.svg",
            "365" => "svg/wi-day-sleet-storm.svg",
            "368" => "svg/wi-day-snow.svg",
            "371" => "svg/wi-day-snow-wind.svg",
            "374" => "svg/wi-day-hail.svg",
            "377" => "svg/wi-day-hail.svg",
            "386" => "svg/wi-day-rain.svg",
            "389" => "svg/wi-day-thunderstorm.svg",
            "392" => "svg/wi-day-snow-thunderstorm.svg",
            "395" => "svg/wi-day-thunderstorm.svg",
            "NA" => "svg/wi-na.svg",
            _ => "svg/wi-na.svg",
        },
        true => match weather_code {
            "113" => "svg/wi-night-clear.svg",
            "116" => "svg/wi-night-partly-cloudy.svg",
            "119" => "svg/wi-night-cloudy.svg",
            "122" => "svg/wi-night-cloudy.svg",
            "143" => "svg/wi-night-fog.svg",
            "176" => "svg/wi-night-rain.svg",
            "179" => "svg/wi-night-snow.svg",
            "182" => "svg/wi-night-sleet.svg",
            "185" => "svg/wi-night-rain-mix.svg",
            "200" => "svg/wi-night-lightning.svg",
            "227" => "svg/wi-night-snow-wind.svg",
            "230" => "svg/wi-night-snow-wind.svg",
            "248" => "svg/wi-night-fog.svg",
            "260" => "svg/wi-night-fog.svg",
            "263" => "svg/wi-night-rain.svg",
            "266" => "svg/wi-night-rain.svg",
            "281" => "svg/wi-night-rain-mix.svg",
            "284" => "svg/wi-night-rain-mix.svg",
            "293" => "svg/wi-night-rain.svg",
            "296" => "svg/wi-night-rain.svg",
            "299" => "svg/wi-night-rain.svg",
            "302" => "svg/wi-night-rain.svg",
            "305" => "svg/wi-night-storm-showers.svg",
            "308" => "svg/wi-night-storm-showers.svg",
            "311" => "svg/wi-night-rain-mix.svg",
            "314" => "svg/wi-night-rain-mix.svg",
            "317" => "svg/wi-night-sleet.svg",
            "320" => "svg/wi-night-sleet-storm.svg",
            "323" => "svg/wi-night-snow.svg",
            "326" => "svg/wi-night-snow.svg",
            "329" => "svg/wi-night-snow.svg",
            "332" => "svg/wi-night-snow.svg",
            "335" => "svg/wi-night-snow.svg",
            "338" => "svg/wi-night-snow.svg",
            "350" => "svg/wi-night-hail.svg",
            "353" => "svg/wi-night-hail.svg",
            "356" => "svg/wi-night-snow-thunderstorm.svg",
            "359" => "svg/wi-night-thunderstorm.svg",
            "362" => "svg/wi-night-sleet.svg",
            "365" => "svg/wi-night-sleet-storm.svg",
            "368" => "svg/wi-night-snow.svg",
            "371" => "svg/wi-night-snow.svg",
            "374" => "svg/wi-night-hail.svg",
            "377" => "svg/wi-night-hail.svg",
            "386" => "svg/wi-night-rain.svg",
            "389" => "svg/wi-night-thunderstorm.svg",
            "392" => "svg/wi-night-snow.svg",
            "395" => "svg/wi-night-snow-thunderstorm.svg",
            "NA" => "svg/wi-na.svg",
            _ => "svg/wi-na.svg",
        },
    };

    let current_directory: std::path::PathBuf = get_current_dir();
    let file_path: std::path::PathBuf = current_directory.join(weather_image);
    let svg_path: String = file_path.to_string_lossy().into_owned();

    svg_path
}

fn is_night(last_time_updated: &str, sunset: &str, force_day: bool) -> bool {
    if force_day {
        return false;
    }

    let updated_time: NaiveTime = match NaiveTime::parse_from_str(last_time_updated, "%I:%M %P") {
            Ok(time) => time,
            Err(_) => NaiveTime::default(),
    };

    let night_time: NaiveTime = match NaiveTime::parse_from_str(sunset, "%I:%M %p") {
        Ok(time) => time,
        Err(_) => NaiveTime::default(),
    };

    if updated_time != NaiveTime::default() && night_time != NaiveTime::default() {
        updated_time > night_time
    } else {
        false
    }
}

fn save_weather_json(json_text: String) -> Result<(), io::Error> {
    let settings_file: std::path::PathBuf = get_current_dir();
    let file_path: std::path::PathBuf = settings_file.join("last_weather.json");
    if file_path.exists() {
        remove_file(&file_path)?;
    }

    let mut file: File = File::create(file_path)?;
    file.write_all(json_text.as_bytes())?;

    Ok(())
}

fn get_daily_weather_code(hourly_weather: Vec<ForecastHour>) -> String {
    let mut weather_codes: HashMap<String, i16> = HashMap::new();
    for hour in hourly_weather {
        let weather_code: String = hour.weather_code;
        *weather_codes.entry(weather_code).or_insert(0) += 1;
    }

    weather_codes.into_iter().max_by_key(|&(_, count)| count).map(|(key, _)| key).expect("No weather codes available.")
}

fn reformat_date(raw_date: String) -> Result<String, ParseError> {
    let proc_date: NaiveDate = NaiveDate::parse_from_str(raw_date.as_str(), "%F")?;

    Ok(proc_date.format("%A %B %e, %Y").to_string())
}

fn reformat_time(raw_time: String, hour_12: bool) -> (String, NaiveTime) {
    let temp_hour: u32 = raw_time.replace("00", "").parse::<u32>().unwrap();
    let proc_time = match NaiveTime::from_hms_opt(temp_hour, 0, 0){
        Some(x) => x,
        None => {
            println!("Error getting time for hourly forecast: {raw_time}");
            NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap()
        }
    };

    let time_format: &str = match hour_12 {
        true => "%I %p",
        false => "%H"
    };

    (proc_time.format(time_format).to_string(), proc_time)
}