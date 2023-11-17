
mod misc;
mod request_weather;
mod ui;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
mod rpi;

use iced::{Application, Settings, window};

use misc::{UserSettings, SettingsError, get_current_dir};
use ui::WeatherGui;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use rpi::wait_for_motion;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use std::thread;

fn main() {
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let _wakeup = thread::Builder::new().name("Screen Control".to_string()).spawn(|| {wait_for_motion()});

    let current_dir: std::path::PathBuf = get_current_dir();

    let file_path: std::path::PathBuf = current_dir.join("settings.json");
    let file_name: &str = file_path.to_str().expect("Invalid settings file.");

    let user_settings: UserSettings = match UserSettings::new(file_name) {
        Ok(settings_data) => settings_data,
        Err(SettingsError::IOError { error_desc }) => panic!("Error opening user settings file.\n {:?}", error_desc),
        Err(SettingsError::SerdeError { error_desc }) => panic!("Error processing user settings file.\n {:?}", error_desc)
    };

    match WeatherGui::run(Settings {
        flags: user_settings,
        window: window::Settings {
        size: (500, 800),
        ..window::Settings::default()
        },
        ..Settings::default()
    }) {
        Ok(()) => (),
        Err(run_gui_error) => panic!("Error running gui: {:?}", run_gui_error)
    };
}



//async fn weather_func() {
//    const ZIP_CODE: &str = "70737";
//    let weather_forecast: Weather = match get_weather(ZIP_CODE.to_string()).await {
//        Ok(weather) => weather,
//        Err(error) => {
//            println!("Error in retrieving the weather:\n{:?}", error);
//            return;
//        }
//    };
//
//    println!("It feels like {}°F",
//        weather_forecast.current_weather.current_condition[0].feels_like_f
//    );
//    println!("The humidity is {}%",
//        weather_forecast.current_weather.current_condition[0].humidity
//    );
//    println!("The pressure is {} hPa",
//        weather_forecast.current_weather.current_condition[0].pressure
//    );
//    println!("Today's forecast: \t\t\tAvg {}°F\tMin {}°F\tMax {}°F",
//        weather_forecast.daily_forecast.weather[0].avg_temp_f,
//        weather_forecast.daily_forecast.weather[0].min_temp_f,
//        weather_forecast.daily_forecast.weather[0].max_temp_f
//    );
//    println!("Tomorrow's forecast: \t\t\tAvg {}°F\tMin {}°F\tMax {}°F",
//        weather_forecast.daily_forecast.weather[1].avg_temp_f,
//        weather_forecast.daily_forecast.weather[1].min_temp_f,
//        weather_forecast.daily_forecast.weather[1].max_temp_f
//    );
//    println!("The day after tomorrow's forecast: \tAvg {}°F\tMin {}°F\tMax {}°F",
//        weather_forecast.daily_forecast.weather[2].avg_temp_f,
//        weather_forecast.daily_forecast.weather[2].min_temp_f,
//        weather_forecast.daily_forecast.weather[2].max_temp_f
//    );
//    println!("Last updated: {} on {}",weather_forecast.last_time_updated, weather_forecast.last_date_updated);
//}
