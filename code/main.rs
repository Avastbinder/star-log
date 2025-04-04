use reqwest::blocking::Client;
use std::error::Error;
use serde_json;
use iced::{Element, Length, Alignment};
use iced::widget::{button, column, row, text, text_input};
use iced;

mod calculations;
pub use crate::calculations::calculate;

// Iced Variables
#[derive(Default)]
struct State{
    latitude: f64,
    longitude: f64,
    altitude: String,
    azimuth: String,
    year: String,
    month: String,
    day: String,
    hour: String,
    minute: String,
    second: String,
    location: String,
    utc_offset: i32,
    brightest_star: [String; 8],
    closest_star: [String; 8],
}

// Iced functions
#[derive(Debug, Clone)]
pub enum Message {
    LocationChanged(String),
    AltitudeChanged(String),
    AzimuthChanged(String),
    YearChanged(String),
    MonthChanged(String),
    DayChanged(String),
    HourChanged(String),
    MinuteChanged(String),
    SecondChanged(String),
    FetchStar
}

impl State {
    // Display updater
    fn update(&mut self, message: Message) -> () {
        match message {
            Message::LocationChanged(location) => {
                self.location = location;
            }
            Message::AltitudeChanged(altitude) => {
                self.altitude = altitude;
            }
            Message::AzimuthChanged(azimuth) => {
                self.azimuth = azimuth;
            }
            Message::YearChanged(year) => {
                self.year = year;
            }
            Message::MonthChanged(month) => {
                self.month = month;
            }
            Message::DayChanged(day) => {
                self.day = day;
            }
            Message::HourChanged(hour) => {
                self.hour = hour;
            }
            Message::MinuteChanged(minute) => {
                self.minute = minute;
            }
            Message::SecondChanged(second) => {
                self.second = second;
            }
    
            Message::FetchStar => {
                let mut epoch: i64 = -1;
                let mut ra = 0.0;
                let mut dec = 0.0;

                // Strings are easier for user input. Convert to proper variable types.
                let (altitude, azimuth, year, month, day, hour, minute, second) = 
                calculate::to_correct(&self.altitude, &self.azimuth, &self.year, &self.month, &self.day, &self.hour, &self.minute, &self.second);

                match calculate::get_epoch(self.utc_offset, year, month, day, hour, minute, second) {
                    Ok(_epoch) => epoch = _epoch,
                    Err(_) => println!("Error getting epoch")
                }

                // Get earth coordinates based on zip code or city name
                match calculate::get_location_data(&self.location, epoch) 
                {
                    Ok((lat, long, offset)) => {
                        self.latitude = lat;
                        self.longitude = long;
                        self.utc_offset = offset;
                    }
                    Err(_) => {println!("Error getting location data")}
                }

                // Convert given values into equatorial coordinates for the SIMBAD TAP Query
                match calculate::horizon_to_equatorial(
                self.latitude, self.longitude, altitude, azimuth, year, month, day, hour, minute, second, self.utc_offset) {
                    Ok((_ra, _dec)) => {
                        ra = _ra;
                        dec = _dec;
                    }
                    Err(_) => {println!("Error converting to equatorial")}
                }

                // Use SIMBAD TAP Query with returned equatorial coordinates
                match get_star(ra, dec) {
                    Ok((_brightest_star, _closest_star)) => {
                        let mut i = 0;
                        while i < 8 {
                            self.brightest_star[i] = _brightest_star[i].clone();
                            self.closest_star[i] = _closest_star[i].clone();
                            i+=1;
                        }
                    }
                    Err(_) => println!("Error getting star")
                }
            }
        }
    }

    // Display view
    fn view(&self) -> Element<Message> {
        // Input side of GUI
        let column_fields = column!
        [
            text("Nearest zip code, city name at observation spot").size(14),
            text_input("", &self.location)
                .on_input(Message::LocationChanged),
            text("Type altitude of star (not height of location)").size(14),
            text_input("", &self.altitude.to_string())
                .on_input(Message::AltitudeChanged),
            text("Type azimuth of star").size(14),
            text_input("", &self.azimuth.to_string())
                .on_input(Message::AzimuthChanged),
            text("Type year at observation").size(14),
            text_input("", &self.year.to_string())
                .on_input(Message::YearChanged),
            text("Type month at observation").size(14),
            text_input("", &self.month.to_string())
                .on_input(Message::MonthChanged),
            text("Type day of observation").size(14),
            text_input("", &self.day.to_string())
                .on_input(Message::DayChanged),
            text("Type hour of observation in 24 hour format").size(14),
            text_input("", &self.hour.to_string())
                .on_input(Message::HourChanged),
            text("Use local time, utc offset is applied automatically").size(12),
            text("Type minute of observation").size(14),
            text_input("", &self.minute.to_string())
                .on_input(Message::MinuteChanged),
            text("Type second of observation").size(14),
            text_input("", &self.second.to_string())
                .on_input(Message::SecondChanged),
            text("If unsure, type 30").size(12),
            button("Enter").on_press(Message::FetchStar),
        ].spacing(5).padding(5).width(Length::FillPortion(1));

        // Output side of GUI
        let column_data = column! 
        [
            text("Brightest star in 3 degree radius around point").size(25),
            text(format!("Name: {}\nDistance from entered point in degrees: {:.3}\nStar Magnitude (Brightness): {}\nRight Ascension: {:.6}\nDeclination: {:.6}\nDistance from earth: {:.6} lightyears\nStar type: {}\n",
                    &self.brightest_star[2], &self.brightest_star[1], &self.brightest_star[0], 
                    &self.brightest_star[3], &self.brightest_star[4], 
                    match &self.brightest_star[5].parse::<f64>() {
                        Ok(value) => ((1000.0 / value)*3.262).to_string(),
                        Err(_) => "N/A".to_string()
                    }, 
                    &self.brightest_star[6])).size(16),
            text("\nClosest star to selected point").size(25),
            text(format!("Name: {}\nDistance from entered point in degrees: {:.3}\nStar Magnitude (Brightness): {}\nRight Ascension: {:.6}\nDeclination: {:.6}\nDistance from earth: {:.6} lightyears\nStar type: {}\n",
                    &self.closest_star[2], &self.closest_star[1], &self.closest_star[0], 
                    &self.closest_star[3], &self.closest_star[4], 
                    match &self.closest_star[5].parse::<f64>() {
                        Ok(value) => ((1000.0 / value)*3.262).to_string(),
                        Err(_) => "N/A".to_string()
                    }, 
                    &self.closest_star[6])).size(16),
            text("\n\nNotes: \n- Requires internet connection\n- Program may take a couple seconds to load the data.\n- A magnitude of 6 is near the limit of what you could see with your naked eye.
- The first letter of the star type corresponds to its color.\n   * For example, O would be a blue star, M would be a red star.
   * From blue to red, the order is: O, B, A, F, G, K, M.\n   * This order also corresponds to its surface tempature, from hottest (O) to coldest (M).\n   * Our own sun is a G type star.").size(12)
        ].spacing(5).padding(5).width(Length::FillPortion(2));

        row![column_fields, column_data]
        .padding(10)
        .spacing(20)
        .align_y(Alignment::Start)
        .into()

    }
}

fn main() {
    iced::run("Star finder - Aidan Vastbinder", State::update, State::view)
    .expect("Failed to start application")
}

fn get_star(ra: f64, dec: f64) -> Result<([String; 8], [String; 8]), Box<dyn Error>>
{
    let client = Client::new();
    let simbad_url = "http://simbad.u-strasbg.fr/simbad/sim-tap/sync";
    let mut search_area = 0.0;
    let mut brightest = (7.0, 0.0, "".to_string());
    let mut closest = (0.0, 5.0, "".to_string());

    while search_area <= 3.0 // Query with a 3 degree radius
    {
        let query = format!(r#"
        SELECT TOP 1
            b.main_id AS star_id, 
            b.ra, 
            b.dec, 
            b.plx_value,
            b.sp_type,
            f.flux AS visual_magnitude
        FROM 
            basic AS b
        JOIN 
            flux AS f ON b.oid = f.oidref
        WHERE 
            f.filter = 'V'
            AND f.flux <= 7
            AND CONTAINS(POINT('ICRS', b.ra, b.dec), CIRCLE('ICRS', {}, {}, {})) = 1
        ORDER BY 
            visual_magnitude ASC
            "#, ra, dec, search_area);

        let params = [
            ("request", "doQuery"),
            ("lang", "ADQL"),
            ("format", "json"),
            ("query", &query),
        ];

        let response = client.post(simbad_url).form(&params).send()?;

        let json: serde_json::Value = response.json()?;

        // Check the star magnitude, compare to previous highest value, replace if higher.
        if let Some(rows) = json["data"].as_array() {
            if let Some(first_row) = rows.first() {
                if let Some(magnitude) = first_row.get(5).and_then(serde_json::Value::as_f64) {
                    if magnitude < brightest.0 {
                        brightest = (magnitude, search_area, json["data"].to_string());
                    }
                    if search_area < closest.1 {
                        closest = (magnitude, search_area, json["data"].to_string())
                    }
                }
            }
        }
        search_area += 0.1;
    }

    let mut brightest_data = ["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];
    let mut closest_data = ["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()];

    // Assign the brightest and closest stars magnitude and distance from the entered point into the array
    brightest_data[0] = brightest.0.to_string();
    brightest_data[1] = brightest.1.to_string();
    closest_data[0] = closest.0.to_string();
    closest_data[1] = closest.1.to_string();

    let bright_data: Vec<&str> = brightest.2.split(',').collect();
    let close_data: Vec<&str> = closest.2.split(',').collect();

    // Assemble an array of the star data
    let mut i = 0;
    while i < 5 {
        brightest_data[i+2] = bright_data[i].to_string();
        closest_data[i+2] = close_data[i].to_string();
        i+=1;
    }

    // Make name of star prettier
    brightest_data[2] = brightest_data[2][2..].to_string();
    closest_data[2] = closest_data[2][2..].to_string();

    println!("{:?}\n{:?}", brightest_data, closest_data);

    Ok((brightest_data, closest_data))
}




