// #[macro_use(defer)] extern crate scopeguard;
use std::{thread::sleep, io::Cursor, io::stdout, time::Duration, process::Command};
use log::{debug, info};
use adb_client::{ADBServer, ADBServerDevice, RustADBError};
use chrono::{DateTime, Datelike, FixedOffset};

macro_rules! ternary {
    ($test:expr , $true_expr:expr , $false_expr:expr) => {
        if $test {$true_expr} else {$false_expr} }}


fn main() {
    log::set_max_level(log::LevelFilter::Info);
    let mut space = Space::new();
    space.cycle(100).unwrap()
}

#[cfg(test)] mod test {
    use super::*;
    #[test] fn run() { Space::new().cycle(100).unwrap() }
    #[test] fn back() { Device::new().unwrap().fake_mode(false).unwrap() }
}

pub struct Space {
    device : Device
}


impl Space {
    pub fn new() -> Space { Space{ device: Device::new().unwrap() } }
    pub fn cycle(&mut self, cnt: u32) -> Result<(), RustADBError> {
        info!("🟢🟢🟢 {cnt}");
        self.device.fake_mode(true)?;
        for i in 0..cnt {
            info!("🟢 {i}");
            self.add_hours(6)?;
            sleep(Duration::from_millis(1900));
            self.device.tap(911,2466)?;
            self.device.tap(931,2178)?;
            sleep(Duration::from_millis(500));
        }
        println!("🟢❇️🟢");
        Command::new("osascript").args(["-e", r#"display notification "DONE" with title "‼️"""#]).
            output()?;
        Ok(())
    }

    fn retrieve_date(&mut self) -> Result<DateTime<FixedOffset>, RustADBError> {
        let s = self.device.exec("date")?;
        let s = "2024 ".to_string() + s.strip_suffix(" 2024\n").unwrap() + ":00";
        let d = DateTime::parse_from_str(s.as_str(), "%Y %a %b %e %H:%M:%S %::z").unwrap();
        Ok(d)
    }

    fn add_hours(&mut self, hours: u64) -> Result<(), RustADBError> {
        self.switch_game_app(false)?;
        sleep(Duration::from_millis(1000));

        let t = self.retrieve_date()?;
        let newt = t + Duration::from_secs(3600u64 * hours);
        if newt.day() != t.day() { // set next day
            // 1109,803 -1504,1078
            let (basex, stepx) = (1109, (1504-1109)/(7-1));
            let (basey, stepy) = (803, (1078-803)/(6-1));
            let day = newt.weekday().number_from_sunday()-1; // col 0 - 6
            //// TODO !!!!!! window can ends on last day of month
            let week = (ternary!(t.month() == newt.month(), newt.day(), t.day())+1) / 7;  // row 0-5
            self.device.tap(1178, 490)?;           //set date
            self.device.tap(basex+day*stepx, basey+week*stepy)?;
            self.device.tap(1450, 1171)?; //  Done
        }

        info!("🟩> {t}");
        self.device.tap(949, 615)?; // set time
        sleep(Duration::from_millis(1300));
        self.device.drag(1188, 1092, 1188, 790)?;
        sleep(Duration::from_millis(100));
        self.device.drag(1188, 1092, 1188, 790)?;
        sleep(Duration::from_millis(100));
        self.device.tap(1456, 1160)?;

        self.switch_game_app(true)
    }

    fn switch_game_app(&mut self, game: bool) -> Result<(), RustADBError> {
        let _s = self.device.exec(("am start ".to_string() + if game {
            "-n com.TironiumTech.IdlePlanetMiner/com.google.firebase.auth.internal.GenericIdpActivity"
        } else {
            "-a android.settings.DATE_SETTINGS"
        }).as_str())?;
        //if _s != "" { println!("{_s}") }
        Ok(())
    }
}


struct Device {
    pub device : ADBServerDevice
}

impl Device {
    pub fn new() -> Result<Device, RustADBError> {
        let mut server = ADBServer::default();
        server.devices()?.iter().for_each(|device| {
            info!("♦️ {}", device.identifier);
        });
        Ok(Self{
            device: server.get_device()?
        })
    }

    pub fn fake_mode(&mut self, fake : bool) -> Result<(), RustADBError> {
        self.device.shell_command(["svc", "wifi", ternary!(fake ,"disable","enable") ],stdout())?;
        self.device.shell_command(["settings", "put", "global", "auto_time", ternary!(fake ,"0","1") ],stdout())?;
        Ok(())
    }

    pub fn tap(&mut self, x: u32, y: u32) -> Result<(), RustADBError> {
        sleep(Duration::from_millis(50));
        self.device.shell_command([format!("input tap {x} {y}")], stdout())?;
        sleep(Duration::from_millis(50));
        debug!("TAP {x}, {y}");
        Ok(())
    }

    pub fn drag(&mut self, x1: u32, y1: u32, x2: u32, y2:u32) -> Result<(), RustADBError> {
        sleep(Duration::from_millis(50));
        self.device.shell_command([format!("input swipe {x1} {y1} {x2} {y2}")], stdout())?;
        sleep(Duration::from_millis(50));
        debug!("DRAG {x1} {y1} {x2} {y2}");
        Ok(())
    }

    pub fn exec(&mut self, command: &str) -> Result<String, RustADBError> {
        let mut buff = Cursor::new(Vec::new());
        self.device.shell_command([command], &mut buff)?;
        Ok(String::from_utf8(buff.into_inner())?)
    }
}

// settings put system pointer_location 0
// 1848 x 2960
