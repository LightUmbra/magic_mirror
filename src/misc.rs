pub use self::errors::{RequestError, UIError, SettingsError};
pub use self::structs::{
    UserSettings, Astronomy, CurrentConditions, CurrentWeather, DailyForecast, ForecastDay, ForecastHour, Weather, get_weather_desc, get_weather_image
};
mod errors;
mod structs;

pub fn get_current_dir() -> std::path::PathBuf {
    let current_dir: std::path::PathBuf = match std::env::current_exe() {
        Ok(x) => x.parent().unwrap().to_owned(),
        Err(pwd_error) => panic!("Can't get currrent directory. {:?}", pwd_error)
    };
    current_dir
}
