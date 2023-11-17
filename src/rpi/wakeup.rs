//#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use rppal::gpio::{Gpio, Pin, Trigger, Error as GpioError};
//#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
use enigo::{Enigo, Key};

fn wait_for_motion() -> Result<(), GpioError> {
    let pi_gpio = Gpio::new().unwrap();
    let motion_pin: InputPin = pi_gpio.get(17)?.into_input();

    let time_delay: Duration = Duration::milliseconds(500);
    let unstarted_timer: DateTime<Local> = DateTime::<Local>::default();

    let mut time_start: DateTime<Local> = unstarted_timer;
    let mut tripped: bool = false;

    loop {
         match motion_pin.read() {
            Level::High if tripped => (),
            Level::High => {
                if time_start != unstarted_timer && (Local::now().signed_duration_since(time_start) >= time_delay) {
                    //println!("Motion Sensor Tripped!");

                    let mut enigo_key = Enigo::new();
                    enigo_key.key_click(Key::Pause);

                    tripped = true;

                 } else if time_start == unstarted_timer {
                    time_start = Local::now();

                }
            },
            Level::Low if tripped => {
                time_start = unstarted_timer;
                tripped = false;
                //println!("Trip Reset.");

            },
            Level::Low => ()
         }
    }

}
