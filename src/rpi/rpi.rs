use rppal::gpio::{Gpio, Level, InputPin};
use chrono::{DateTime, Local, Duration};
use x11rb::{protocol::dpms, rust_connection::RustConnection};

use std::fmt;

pub fn wait_for_motion() {
    let pi_gpio = Gpio::new().unwrap();

    const MOTION_PIN: u8 = 17;
    const MOTION_TIME: i64 = 500;
    const BUTTON_PIN: u8 = 10;
    const BUTTON_TIME: i64 = 75;

    let mut screen_status: DPMSMode = DPMSMode::On;
    let mut screen_time: DateTime<Local> = Local::now();
    let screen_off_time: Duration = Duration::milliseconds(180000);

    let (conn, _) = x11rb::connect(Some(":0")).unwrap();

    let mut motion_events: PinEvent = match PinEvent::new(&pi_gpio, MOTION_PIN, MOTION_TIME, motion_trip, &conn) {
        Ok(x) => x,
        Err(error) => {
            println!("{:?}", error.message);
            match PinEvent::new(&pi_gpio, MOTION_PIN, MOTION_TIME, motion_trip, &conn) {
                Ok(x) => x,
                Err(error) => {panic!("{:?}", error.message);}
            }
        }
    };
    let mut button_events: PinEvent = match PinEvent::new(&pi_gpio, BUTTON_PIN, BUTTON_TIME, button_trip, &conn) {
        Ok(x) => x,
        Err(error) => {
            println!("{:?}", error.message);
            match PinEvent::new(&pi_gpio, BUTTON_PIN, BUTTON_TIME, button_trip, &conn) {
                Ok(x) => x,
                Err(error) => {panic!("{:?}", error.message);}
            }
        }
    };

    loop {
        screen_status = match motion_events.check_trip(&mut screen_status) {
            Ok(x) => x,
            Err(error) => {
                println!("Motion sensor error: {:?}", error.message);
                screen_status
            }
        };
        screen_status = match button_events.check_trip(&mut screen_status) {
            Ok(x) => x,
            Err(error) => {
                println!("button error: {:?}", error.message);
                screen_status
            }
        };

        if !motion_events.tripped && !button_events.tripped && Local::now().signed_duration_since(screen_time) >= screen_off_time && screen_status == DPMSMode::On {
            if let Err(error) = change_screen_mode(&conn, DPMSMode::Off) {
                panic!("{:?}", error.message);
            };
            screen_status = DPMSMode::Off;
        } else if (motion_events.tripped || button_events.tripped) && screen_status == DPMSMode::On {
            screen_time = Local::now();
        }
    };
}

fn change_screen_mode(connection: &RustConnection, mode: DPMSMode) -> Result<(), RPIError> {
    let mode = mode.to_x11();
    let reply = dpms::force_level(connection, mode).unwrap();
    match reply.check(){
        Ok(()) => Ok(()),
        Err(screen_error) => Err(RPIError::new(format!("Can't change screen to {:?}: {:?}", mode, screen_error)))
    }
}

fn motion_trip(connection: &RustConnection , _mode: &DPMSMode) -> Result<DPMSMode, RPIError> {
    let _ = change_screen_mode(connection, DPMSMode::On)?;
    Ok(DPMSMode::On)
}

fn button_trip(connection: &RustConnection, current_mode: &DPMSMode) -> Result<DPMSMode, RPIError> {
    let temp_mode = current_mode.toggle();
    change_screen_mode(connection, temp_mode)?;
    Ok(temp_mode)
   
}

#[derive(PartialEq, Copy, Clone)]
enum DPMSMode {
    On,
    Off
}

impl DPMSMode {
    fn to_x11(self) -> dpms::DPMSMode {
        match self {
            DPMSMode::On => dpms::DPMSMode::ON,
            DPMSMode::Off => dpms::DPMSMode::OFF
        }
    }

    fn toggle(self) -> DPMSMode {
        match self {
            DPMSMode::On => DPMSMode::Off,
            DPMSMode::Off => DPMSMode::On
        }
    }
} 

struct  PinEvent<'a> {
    time_start: DateTime<Local>,
    time_delay: Duration,
    tripped: bool,
    pin: InputPin,
    trip_action: for<'c, 'b> fn(&'c RustConnection, &'b DPMSMode) -> Result<DPMSMode, RPIError>,
    connection: &'a RustConnection
}

impl PinEvent<'_> {
    fn new<'a> (gpio: &'a Gpio, pin_num: u8, time_delay: i64, trip_action: for<'c, 'b> fn(&'c RustConnection, &'b DPMSMode) -> Result<DPMSMode, RPIError>, connection: &'a RustConnection) -> Result<PinEvent<'a>, RPIError> {
        let gpio_pin: InputPin = match gpio.get(pin_num) {
            Ok(x) => x.into_input(),
            Err(gpio_error) => {return Err(RPIError::new(format!("Wakeup Error On pin {}: {:?}", pin_num, gpio_error)));}
        };

        let new_pin = PinEvent {
            time_start: DateTime::<Local>::default(),
            time_delay: Duration::milliseconds(time_delay),
            tripped: false,
            trip_action,
            pin: gpio_pin,
            connection
        };

        Ok(new_pin)
    }

    fn check_trip (&mut self, current_mode: &DPMSMode) -> Result<DPMSMode, RPIError> {
        let unstarted_timer = DateTime::<Local>::default();

        match self.pin.read() {
            Level::High if self.tripped => (),
            Level::High => {
                if self.time_start != unstarted_timer && (Local::now().signed_duration_since(self.time_start) >= self.time_delay) {
                    
                    let mode = (self.trip_action)(self.connection, current_mode)?;
                    self.tripped = true;
                    return Ok(mode);

                } else if self.time_start == unstarted_timer {
                    self.time_start = Local::now();
                }
            },
            Level::Low => { // if self.tripped => {
                self.time_start = unstarted_timer;
                self.tripped = false;

            },
            //Level::Low => ()
        }
        return Ok(*current_mode);
    }
}

#[derive(Debug, Clone)]
struct RPIError{
    message: String
}

impl fmt::Display for RPIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.message)
    }
}

impl RPIError {
    fn new(message: String) -> RPIError {
        RPIError {message: message}
    }
}

