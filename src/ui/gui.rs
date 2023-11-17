use iced::event::{self, Event};
use iced::{subscription, Renderer, keyboard, theme, executor};
use iced::widget::{column, container, row, text, button, svg, horizontal_rule, vertical_rule};
use iced::{Alignment, Application, Command, Element, Length, Theme, Subscription, window, color, Padding};
use iced_native::{command, window as window_action}; // Soon to be iced_runtime
use chrono::Local;

use crate::misc::{Weather, UserSettings, ForecastDay, ForecastHour, UIError, get_weather_image};
use crate::request_weather::get_weather;

pub struct WeatherGui {
    pub status: WeatherGuiStatus,
    pub zip_code: String,
    pub unit: String,
    pub hour_12: bool,
    pub weather_state: WeatherShim,
}   

#[derive(Debug, Clone)]
pub enum WeatherGuiStatus {
    Loading,
    Loaded { weather: WeatherShim },
    Errored { error: UIError },
}

#[derive(Debug, Clone, Default)]
pub struct WeatherShim {
    pub current_weather: CurrentWeatherGui,
    pub daily_weather: Vec<DaysWeatherGui>,
    pub hourly_weather: Vec<HourlyWeatherGui>,
    pub clock: String,
    pub date: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Update,
    TickClock,
    WeatherUpdated(Result<WeatherShim, UIError>),
    ToggleFullscreen(window::Mode),
}

#[derive(Debug, Clone)]
pub struct CurrentWeatherGui {
    pub last_time_updated: String,
    pub last_date_updated: String,
    pub unit: String,
    pub current_temp: String,
    pub feels_like: String,
    pub humidity: String,
    pub uv_index: String,
    pub visibility: String,
    pub weather_desc: String,
    pub weather_image: svg::Handle,
}

impl Default for CurrentWeatherGui {
    fn default() -> Self {
        CurrentWeatherGui { 
            last_time_updated: "".to_string(),
            last_date_updated: "".to_string(),
            unit: "".to_string(),
            current_temp: "".to_string(),
            feels_like: "".to_string(),
            humidity: "".to_string(),
            uv_index: "".to_string(),
            visibility: "".to_string(),
            weather_desc: "Error".to_string(),
            weather_image: svg::Handle::from_path("svg\\wi-na.svg")
        }
    }
}

#[derive(Debug, Clone)]
pub struct DaysWeatherGui {
    pub unit: String,
    pub date: String,
    pub max_temp: String,
    pub min_temp: String,
    pub uv_index: String,
    pub sunrise: String,
    pub sunset: String,
    pub average_chance_of_precip: String,
    pub weather_desc: String,
    pub weather_image: svg::Handle,
}

#[derive(Debug, Clone)]
pub struct HourlyWeatherGui  {
    pub unit: String, 
    pub time: String,
    pub temp: String,
    pub feels_like: String,
    pub chance_of_precip: String,
    pub weather_desc: String,
    pub weather_image: svg::Handle,
} 

impl Application for WeatherGui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = UserSettings;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let gui_startup = WeatherGui {
            zip_code: flags.zip_code.clone(),
            unit: flags.unit.clone(),
            hour_12: flags.hour_12,
            status: WeatherGuiStatus::Loading,
            weather_state: WeatherShim::default()
        };
        
        let fullscreening = Command::single(
            iced_native::command::Action::Window(iced_native::window::Action::ChangeMode(window::Mode::Fullscreen))
        );
        
        let startup = Command::perform(update_all_weather(flags.zip_code.clone(), flags.unit.clone(), flags.hour_12), Message::WeatherUpdated);

        ( 
        gui_startup,
        Command::batch([fullscreening, startup])
        )
    }

    fn title(&self) -> String {
        "Magic Mirror".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WeatherUpdated(Ok(weather)) => {
                self.weather_state = weather.clone();
                self.status = WeatherGuiStatus::Loaded { weather };
                Command::none()
            },

            Message::WeatherUpdated(Err(weather_error)) => {
                dbg!(&weather_error);
                self.status = WeatherGuiStatus::Errored { error: weather_error };
                Command::none()
            },

            Message::Update => {
                Command::perform(update_all_weather(self.zip_code.clone(), self.unit.clone(), self.hour_12), Message::WeatherUpdated)
            },

            Message::TickClock => {
                match &self.status {
                    WeatherGuiStatus::Loaded { weather } => {
                        self.weather_state.clock = get_clock(&self.hour_12);
                        self.status = WeatherGuiStatus::Loaded { weather: weather.clone() };
                    },
                    _ => ()
                } 
                Command::none()
            },
            
            Message::ToggleFullscreen(mode) => {
                Command::single(command::Action::Window(window_action::Action::ChangeMode(mode)))
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let key_commands: iced_futures::Subscription<iced_native::Hasher, (event::Event, event::Status), Message> =
         subscription::events_with(|event: Event, status: event::Status| match (event, status){
            (Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: keyboard::KeyCode::Escape,
                modifiers: _,
                }), event::Status::Ignored,
            ) => Some(Message::ToggleFullscreen(window::Mode::Windowed)),
            
            (Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: keyboard::KeyCode::F11,
                modifiers: _,
                }), event::Status::Ignored,
            ) => Some(Message::ToggleFullscreen(window::Mode::Fullscreen)),
            
            (Event::Keyboard(keyboard::Event::KeyPressed { 
                key_code: keyboard::KeyCode::F5,
                modifiers: _,
               }), event::Status::Ignored,
            ) => Some(Message::Update),

           _ => None
        });

        let timer_update: iced_futures::Subscription<_, _, Message> = iced::time::every(std::time::Duration::from_secs(1800)).map(|_| {Message::Update});
        let clock_update: iced_futures::Subscription<_, _, Message> = iced::time::every(std::time::Duration::from_secs(3)).map(|_| {Message::TickClock});

        Subscription::batch([key_commands, timer_update, clock_update])
    }

    fn view(&self) -> Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = match self.status.clone() {
            WeatherGuiStatus::Loading => {
                column![text("Updating Weather...").size(40),].width(Length::Shrink)
            },

            WeatherGuiStatus::Loaded {weather} => {
                let left_side = column![
                    row![
                        weather.current_weather.view(),
                    ].align_items(Alignment::Center)
                    .height(Length::FillPortion(5)),
                    horizontal_rule(25),
                    row![
                        column![
                            row![
                                weather.hourly_weather[0].clone().view(),
                                weather.hourly_weather[1].clone().view(),
                                weather.hourly_weather[2].clone().view(),
                            ].spacing(20)
                            .align_items(Alignment::Center)
                            .height(Length::FillPortion(1)),
                            horizontal_rule(25),
                            row![
                                weather.hourly_weather[3].clone().view(),
                                weather.hourly_weather[4].clone().view(),
                                weather.hourly_weather[5].clone().view(),
                            ].spacing(20)
                            .align_items(Alignment::Center)
                            .height(Length::FillPortion(1)),
                            horizontal_rule(25),
                            row![
                                weather.hourly_weather[6].clone().view(),
                                weather.hourly_weather[7].clone().view(),
                            ].spacing(20)
                            .align_items(Alignment::Center)
                            .height(Length::FillPortion(1)),
                        ].spacing(20)
                    ].height(Length::FillPortion(12))
                ].max_width(500)
                .spacing(20)
                .align_items(Alignment::Center);

                let right_side = column![
                    weather.daily_weather[0].clone().view(),
                    horizontal_rule(25),
                    weather.daily_weather[1].clone().view(),
                    horizontal_rule(25),
                    weather.daily_weather[2].clone().view(),
                ]
                .max_width(500)
                .spacing(20)
                .align_items(Alignment::Start);

                column![
                    row![
                        text(format!("{} - {}", weather.clock, weather.date)).size(30),
                    ]
                    .spacing(20).padding(Padding{
                                             top: 10.0,
                                             right: 0.0,
                                             bottom: 0.0,
                                             left: 0.0
                                         }),

                    horizontal_rule(25),

                    row![
                        left_side,
                        vertical_rule(25),
                        right_side,
                    ],
                ].align_items(Alignment::Center)
            },

            WeatherGuiStatus::Errored { error } => {
                let error_string: String = match error {
                    UIError::APIError { msg } => msg,
                    UIError::DataError { msg } => msg
                };

                column![
                    text("Whoops! Something went wrong...").size(40),
                    text(error_string).size(30),
                    button("Try again").on_press(Message::Update)
                ]
                .spacing(20)
                .align_items(Alignment::Center)
            }
        };

        return container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
    }

    fn theme(&self) -> Self::Theme {
        let custom_pallet: theme::Palette = theme::Palette{
            background: color!(0, 0, 0),
            text: color!(255, 255, 255),
            primary: color!(127, 127, 127),
            danger: color!(191, 191, 191),
            success: color!(0, 128, 0)
        };
        let custom_theme: Box<theme::Custom> = Box::<theme::Custom>::new(theme::Custom::new(custom_pallet));
        Theme::Custom(custom_theme)
    }
}

impl CurrentWeatherGui {
    fn view<'a>(self) -> Element<'a, Message> {
        let svg: iced::widget::Svg<Renderer> = svg(self.weather_image.clone());
            //.width(200) //100
            //.height(200);//Length::FillPortion(2)) //100
            //.content_fit(ContentFit::Contain);

        let current_row = row![
            column![
                svg.height(Length::FillPortion(3)),
                text(&self.weather_desc).size(30),
                text(format!("{1}°{0}    Feels like {2}°{0}", self.unit ,self.current_temp, self.feels_like)).size(30),
                text(format!("Humidity: {}%", self.humidity)).size(30),
                text(format!("Visibility: {}", self.visibility)).size(30), //text(format!("Visibility: {}    UV Index: {}", self.visibility, self.uv_index)).size(30),
                text(format!("UV Index: {}", self.uv_index)).size(30),
                text(format!("Time: {}", Local::now().format("%I:%M %P"))).size(30),
            ].spacing(10)
            .align_items(Alignment::Center)
        ];

        return container(current_row)
            .width(Length::Fill)
            .height(Length::Shrink)
            .center_x()
            .center_y()
            .into();
    }

    async fn update(weather_forecast: &Weather, unit: &str) -> Result<CurrentWeatherGui, UIError> {

        let current_weather: CurrentWeatherGui = match unit.to_lowercase().as_str() { 
            "f" => CurrentWeatherGui {
                last_time_updated: weather_forecast.last_time_updated.clone(),
                last_date_updated: weather_forecast.last_date_updated.clone(),
                unit: unit.to_string(),
                current_temp: weather_forecast.current_weather.current_condition[0].temp_f.clone(),
                feels_like: weather_forecast.current_weather.current_condition[0].feels_like_f.clone(),
                humidity: weather_forecast.current_weather.current_condition[0].humidity.clone(),
                visibility: weather_forecast.current_weather.current_condition[0].visibility.clone(),
                uv_index: weather_forecast.current_weather.current_condition[0].uv_index.clone(),
                weather_desc: weather_forecast.current_weather.current_condition[0].weather_desc.clone(),
                weather_image: svg::Handle::from_path(weather_forecast.current_weather.current_condition[0].weather_image.clone()),
            },
            "c" => CurrentWeatherGui {
                last_time_updated: weather_forecast.last_time_updated.clone(),
                last_date_updated: weather_forecast.last_date_updated.clone(),
                unit: unit.to_string(),
                current_temp: weather_forecast.current_weather.current_condition[0].temp_c.clone(),
                feels_like: weather_forecast.current_weather.current_condition[0].feels_like_c.clone(),
                humidity: weather_forecast.current_weather.current_condition[0].humidity.clone(),
                visibility: weather_forecast.current_weather.current_condition[0].visibility.clone(),
                uv_index: weather_forecast.current_weather.current_condition[0].uv_index.clone(),
                weather_desc: weather_forecast.current_weather.current_condition[0].weather_desc.clone(),
                weather_image: svg::Handle::from_path(weather_forecast.current_weather.current_condition[0].weather_image.clone()),
            },
            _ => CurrentWeatherGui {
                last_time_updated: "".to_string(),
                last_date_updated: "".to_string(),
                unit: "".to_string(),
                current_temp: "Error".to_string(),
                feels_like: "".to_string(),
                humidity: "".to_string(),
                visibility: "".to_string(),
                uv_index: "".to_string(),
                weather_desc: "".to_string(),
                weather_image: svg::Handle::from_path(".\\src\\svg\\wi-na.svg"),
            }
        };

        Ok(current_weather)
    }
}

impl DaysWeatherGui {
    fn view<'a>(self) -> Element<'a, Message> {
        let svg: iced::widget::Svg<Renderer> = svg(self.weather_image.clone());
            //.width(100)
            //.height(100);

        let day_row = row![
            column![
                svg.height(Length::FillPortion(4)),
                text(self.date).size(30).height(Length::Fill),
                text(self.weather_desc).size(30).height(Length::Fill),
                text(format!("High: {1}°{0}    Low: {2}°{0}", self.unit, self.max_temp, self.min_temp)).size(30).height(Length::Fill),
                text(format!("{}% chance of precipitation", self.average_chance_of_precip)).size(30).height(Length::Fill),
                text(format!("Sunrise: {}    Sunset: {}", self.sunrise, self.sunset)).size(30).height(Length::Fill),
            ].spacing(10)
            .align_items(Alignment::Center),
        ];

        return container(day_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
    }

    async fn update(day_weather: &ForecastDay, unit: &str) -> Result<DaysWeatherGui, UIError> {
        let average_precip: String = match get_average_precip(&day_weather.hourly) {
            Ok(x) => format!("{:.0}", x),
            Err(_) => "Error".to_string()
        };
        let current_weather: DaysWeatherGui = match unit.to_lowercase().as_str() { 
            "f" => DaysWeatherGui {
                unit: unit.to_string(),
                date: day_weather.date.to_string(),
                max_temp: day_weather.max_temp_f.to_string(),
                min_temp: day_weather.min_temp_f.to_string(),
                uv_index: day_weather.uv_index.to_string(),
                sunrise: day_weather.astronomy[0].sunrise.clone(),
                sunset: day_weather.astronomy[0].sunset.clone(),
                average_chance_of_precip: average_precip,

                weather_desc: day_weather.weather_desc.to_string(),
                weather_image: svg::Handle::from_path(&day_weather.weather_image),
            },
            "c" => DaysWeatherGui {
                unit: unit.to_string(),
                date: day_weather.date.to_string(),
                max_temp: day_weather.max_temp_c.to_string(),
                min_temp: day_weather.min_temp_c.to_string(),
                uv_index: day_weather.uv_index.to_string(),
                sunrise: day_weather.astronomy[0].sunrise.clone(),
                sunset: day_weather.astronomy[0].sunset.clone(),
                average_chance_of_precip: average_precip,

                weather_desc: day_weather.weather_desc.to_string(),
                weather_image: svg::Handle::from_path(&day_weather.weather_image),
            },
            _ => DaysWeatherGui {
                unit: "".to_string(),
                date: "Error".to_string(),
                max_temp: "".to_string(),
                min_temp: "".to_string(),
                uv_index: "".to_string(),
                sunrise: "".to_string(),
                sunset: "".to_string(),
                average_chance_of_precip: "".to_string(),
                weather_desc: "Error".to_string(),
                weather_image: svg::Handle::from_path(get_weather_image("NA", 
                                                                        "12:00 PM", 
                                                                        "9:00 PM",
                                                                        false)),
            }
        };
        Ok(current_weather)
    }
}

impl HourlyWeatherGui {
    fn view<'a>(self) -> Element<'a, Message> {
        let svg: iced::widget::Svg<Renderer> = svg(self.weather_image.clone());
            //.width(100)
            //.height(100);

        let day_row = 
            column![
                text(self.time).size(30).height(Length::FillPortion(1)),
                svg.height(Length::FillPortion(5)),
                text(self.weather_desc).size(22).height(Length::FillPortion(1)),
                text(format!("{1}°{0} Feels like: {2}°{0}", self.unit, self.temp, self.feels_like)).size(21).height(Length::FillPortion(1)),
                text(format!("{}% chance of precip", self.chance_of_precip)).size(21).height(Length::FillPortion(1)),
            ].spacing(10)
            .align_items(Alignment::Center);

        return container(day_row)
            .width(Length::Fill)
            .height(Length::Fill)//Length::Fixed(183.0)) //Length::FillPortion(1))
            .center_x()
            .center_y()
            .into();
    }

    async fn update(hour_weather: &ForecastHour, unit: &str) -> Result<HourlyWeatherGui, UIError> {
        let rain_chance: f32 = match hour_weather.chance_of_rain.parse::<f32>() {
            Ok(x) => x,
            Err(rain_error) => {return Err(UIError::DataError {msg: format!("Error parsing rain chance {:?}", rain_error)});}
        };
        let snow_chance: f32 = match hour_weather.chance_of_snow.parse::<f32>() {
            Ok(x) => x,
            Err(snow_error) => {return Err(UIError::DataError {msg: format!("Error parsing rain chance {:?}", snow_error)});}
        }; 
        let precip: String = (rain_chance + snow_chance).to_string();
        
        let current_weather: HourlyWeatherGui = match unit.to_lowercase().as_str() { 
            "f" => HourlyWeatherGui {
                unit: unit.to_string(),
                time: hour_weather.time.to_string(),
                temp: hour_weather.temp_f.to_string(),
                feels_like: hour_weather.feels_like_f.to_string(),
                chance_of_precip: precip,
                weather_desc: hour_weather.weather_desc.to_string(),
                weather_image: svg::Handle::from_path(&hour_weather.weather_image)
            },
            "c" => HourlyWeatherGui {
                unit: unit.to_string(),
                time: hour_weather.time.to_string(),
                temp: hour_weather.temp_c.to_string(),
                feels_like: hour_weather.feels_like_c.to_string(),
                chance_of_precip: precip,
                weather_desc: hour_weather.weather_desc.to_string(),
                weather_image: svg::Handle::from_path(&hour_weather.weather_image)
            },
            _ => HourlyWeatherGui {
                unit: "".to_string(),
                time: "".to_string(),
                temp: "".to_string(),
                feels_like: "".to_string(),
                chance_of_precip: "".to_string(),
                weather_desc: "Error".to_string(),
                weather_image: svg::Handle::from_path(get_weather_image("NA", 
                                                                        "12:00 PM", 
                                                                        "9:00 PM",
                                                                        false)),
            }
        };
        Ok(current_weather)
    }
}

fn get_average_precip(hourly_weather: &Vec<ForecastHour>) -> Result<f32, UIError>  {
    let mut precip_chances: f32 = 0.0;
    for hour in hourly_weather {
        let rain_chance: f32 = match hour.chance_of_rain.parse::<f32>() {
            Ok(x) => x,
            Err(rain_error) => {return Err(UIError::DataError {msg: format!("Error parsing rain chance {:?}", rain_error)});}
        };
        let snow_chance: f32 = match hour.chance_of_snow.parse::<f32>() {
            Ok(x) => x,
            Err(snow_error) => {return Err(UIError::DataError {msg: format!("Error parsing rain chance {:?}", snow_error)});}
        };

        precip_chances += rain_chance + snow_chance;
    }

    Ok(precip_chances / hourly_weather.len() as f32)
}

async fn update_all_weather(zip_code: String, unit: String, hour_12: bool) -> Result<WeatherShim, UIError> {
    let weather_forecast: Weather = match get_weather(&zip_code, &hour_12).await {
        Ok(x) => x,
        Err(req_error) => {return Err(UIError::APIError {msg: format!("Error in getting weather forecast: {:?} at {:?}", req_error, req_error.source())});}
    };

    let current_weather = match CurrentWeatherGui::update(&weather_forecast, &unit).await {
        Ok(x) => x,
        Err(curr_err) => {return Err(UIError::DataError {msg: format!("Error in getting current weather. {:?}", curr_err)});}
    }; 

    let mut daily_weather: Vec<DaysWeatherGui> = Vec::new();
    for day in &weather_forecast.daily_forecast.weather {
        let temp =  match DaysWeatherGui::update(day, &unit).await {
            Ok(x) => x,
            Err(day_err) => {return Err(UIError::DataError {msg: format!("Error in getting day {}'s weather. {:?}", day.date, day_err)});}
        };

        daily_weather.push(temp);
    }

    let mut hourly_weather: Vec<HourlyWeatherGui> = Vec::new();
    for hour in &weather_forecast.daily_forecast.weather[0].hourly {
        let temp = match HourlyWeatherGui::update(hour, &unit).await {
            Ok(x) => x,
            Err(hour_err) => {return Err(UIError::DataError {msg: format!("Error in getting hour {}'s weather. {:?}", hour.time, hour_err)});}
        };

        hourly_weather.push(temp);
    }

    Ok(WeatherShim {
        current_weather,
        daily_weather,
        hourly_weather,
        clock: get_clock(&hour_12),
        date: get_date()
    })
    
}

fn get_clock(hour_12: &bool) -> String {
    if *hour_12 {
        return Local::now().format("%I:%M %p").to_string();
    } else {
        return Local::now().format("%H:%M").to_string();
    }
}

fn get_date() -> String {
    return Local::now().format("%A %B, %e %Y").to_string();
}
