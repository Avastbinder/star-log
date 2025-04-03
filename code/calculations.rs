pub mod calculate {
    use reqwest;
    use serde_json;
    use std::fs;
    use num_traits::ToPrimitive;
    use std::f64::consts::PI;
    use chrono::{NaiveDateTime, TimeZone, Utc};

    pub fn get_epoch(utc_offset: i32, mut year: i32, mut month: i32, mut day: i32, mut hour: i32, mut minute: i32, mut second: i32) -> Result<i64, Box<dyn std::error::Error>> {
        match utc_offset_accountance(utc_offset, year, month, day, hour, minute, second)
        {
            Ok((y, m, d, h, min, s)) => {
                year = y;
                month = m;
                day = d;
                hour = h;
                minute = min;
                second = s;
            }
            Err(_) => {println!("Error converting date to utc")}
        }

        // Formats the date for NaiveDateTime
        let input_time = format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second);

        let naive_dt = NaiveDateTime::parse_from_str(&input_time, "%Y-%-m-%-d %-H:%-M:%-S")
            .expect("Invalid date format");
        
        let datetime_utc = Utc.from_utc_datetime(&naive_dt);
        
        let epoch = datetime_utc.timestamp();

        Ok(epoch)
    }

    pub fn get_location_data(location: &str, epoch: i64) -> Result<(f64, f64, i32), Box<dyn std::error::Error>> {
        let key = ""; // key deleted for privacy concerns. Run from the .exe
        let url = format!("http://api.weatherapi.com/v1/current.json?key={}&q={}", key, location);
        
        let response = reqwest::blocking::get(&url)?.text()?;
        fs::write("./src/location_data.json", &response)?;
        
        let file = fs::File::open("./src/location_data.json")?;
        let json: serde_json::Value = serde_json::from_reader(file)?;
        
        let lat = json["location"]["lat"].as_f64().ok_or("Missing latitude")?;
        let lon = json["location"]["lon"].as_f64().ok_or("Missing longitude")?;

        let key2 = ""; // key deleted for privacy concerns. Run from the .exe

        let url = format!(" https://maps.googleapis.com/maps/api/timezone/json?location={}%2C{}&timestamp={}&key={}", lat, lon, epoch, key2);

        let response = reqwest::blocking::get(&url)?.text()?;
        fs::write("./src/timezone.json", &response)?;
            
        let file = fs::File::open("./src/timezone.json")?;
        let json: serde_json::Value = serde_json::from_reader(file)?;

        let timezone_offset = json["rawOffset"].as_f64().unwrap();
        let dst_offset = json["dstOffset"].as_f64().unwrap();

        let tz_offset = ((timezone_offset + dst_offset) / 3600.0).round().to_i32().unwrap();
        
        Ok((lat, lon, tz_offset))
    }


    pub fn horizon_to_equatorial(
        latitude: f64,
        longitude: f64,
        altitude: f64,
        azimuth: f64,
        mut year: i32,
        mut month: i32,
        mut day: i32,
        mut hour: i32,
        mut minute: i32,
        mut second: i32,
        utc_offset: i32
    ) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        
        match utc_offset_accountance(utc_offset, year, month, day, hour, minute, second) {
            Ok((y, m, d, h, min, s)) => {
                year = y;
                month = m;
                day = d;
                hour = h;
                minute = min;
                second = s;
            }
            Err(_) => return Err("Error converting date to UTC".into()),
        }
        
        let lat_rad = latitude.to_radians();
        let alt_rad = altitude.to_radians();
        let az_rad = azimuth.to_radians();
        
        // Compute Julian Day and GMST
        let jd = julian_day(year, month, day, hour, minute, second);
        let gmst_deg = gmst(jd);
        println!("\nJulian Day: {}", jd);
        println!("GMST (deg): {}", gmst_deg);
        
        // Compute Local Sidereal Time (LST)
        let lst_deg = (((gmst_deg / 15.0) + (longitude / 15.0)) * 15.0).rem_euclid(360.0);
        println!("LST (deg): {}", lst_deg);
        
        // Calculate declination (Dec)
        let sin_dec = alt_rad.sin() * lat_rad.sin() + alt_rad.cos() * lat_rad.cos() * az_rad.cos();
        let dec_rad = sin_dec.asin();
        
        // Compute Hour Angle (H)
        let dec_deg = dec_rad.to_degrees();
        
        let sin_dec = altitude.to_radians().sin() * latitude.to_radians().sin() 
            + altitude.to_radians().cos() * latitude.to_radians().cos() * azimuth.to_radians().cos();
        let dec = sin_dec.asin().to_degrees();
        
        let sin_lha = (-azimuth.to_radians().sin() * altitude.to_radians().cos()) / dec.to_radians().cos();
        let cos_lha = (altitude.to_radians().sin() - (latitude.to_radians().sin() * dec.to_radians().sin()))
            / (dec.to_radians().cos() * latitude.to_radians().cos());
        let lha = sin_lha.atan2(cos_lha).to_degrees().rem_euclid(360.0);
    
        println!("LHA: {}", lha);
        
        let ra_deg = (lst_deg - lha).rem_euclid(360.0);
    
        let (ra, dec) = jnow_to_j2000(ra_deg, dec_deg, year.to_f64().unwrap());
        
        println!("RA: {}\nDEC: {}", ra.rem_euclid(360.0), dec.rem_euclid(360.0));

        Ok((ra.rem_euclid(360.0), dec.rem_euclid(360.0)))
    }

    pub fn to_correct(    
    altitude: &String,
    azimuth: &String,
    year: &String,
    month: &String,
    day: &String,
    hour: &String,
    minute: &String,
    second: &String) -> (f64, f64, i32, i32, i32, i32, i32, i32)
    {
        let mut new_alt = 0.0;
        let mut new_azi = 0.0;
        let mut new_year = 0;
        let mut new_month = 0;
        let mut new_day = 0;
        let mut new_hour = 0;
        let mut new_min = 0;
        let mut new_sec = 0;

        match altitude.parse::<f64>() {
            Ok(val) => new_alt = val,
            Err(_) => {}
        }
        match azimuth.parse::<f64>() {
            Ok(val) => new_azi = val,
            Err(_) => {}
        }
        match year.parse::<i32>() {
            Ok(val) => new_year = val,
            Err(_) => {}
        }
        match month.parse::<i32>() {
            Ok(val) => new_month = val,
            Err(_) => {}
        }
        match day.parse::<i32>() {
            Ok(val) => new_day = val,
            Err(_) => {}
        }
        match hour.parse::<i32>() {
            Ok(val) => new_hour = val,
            Err(_) => {}
        }
        match minute.parse::<i32>() {
            Ok(val) => new_min = val,
            Err(_) => {}
        }
        match second.parse::<i32>() {
            Ok(val) => new_sec = val,
            Err(_) => {}
        }

        (new_alt, new_azi, new_year, new_month, new_day, new_hour, new_min, new_sec)
    }

    /// Compute Julian Day (JD) given a date and time in UTC.
    fn julian_day(year: i32, month: i32, day: i32, hour: i32, minute: i32, second: i32) -> f64 {
        let y = if month <= 2 { year - 1 } else { year };
        let m = if month <= 2 { month + 12 } else { month };
        let a = (y as f64 / 100.0).floor();
        let b = 2.0 - a + (a / 4.0).floor();
        let day_fraction = (hour as f64 + minute as f64 / 60.0 + second as f64 / 3600.0) / 24.0;
        let jd = (365.25 * (y as f64 + 4716.0)).floor()
            + (30.6001 * ((m + 1) as f64)).floor()
            + day as f64
            + b
            - 1524.5
            + day_fraction;
        jd
    }

    // Convert coordinates to where they were in the year 2000, which is what the database uses
    fn jnow_to_j2000(ra: f64, dec: f64, epoch: f64) -> (f64, f64) {
        let (d2r, r2d, t) = (PI / 180.0, 180.0 / PI, (epoch - 2000.0) / 100.0);
        let (zeta, z, theta) = (0.6406161 + 0.0000839 * t, 0.6406161 + 0.0003041 * t, 0.5567530 - 0.0001185 * t);
        let (zeta, _z, theta) = (zeta * t / 3600.0 * d2r, z * t / 3600.0 * d2r, theta * t / 3600.0 * d2r);
        
        let (ra, dec) = (ra * d2r, dec * d2r);
        let (x, y, z) = (ra.cos() * dec.cos(), ra.sin() * dec.cos(), dec.sin());
        let (x, z) = (x * theta.cos() + z * theta.sin(), -x * theta.sin() + z * theta.cos());
        let (x, y) = (x * zeta.cos() - y * zeta.sin(), x * zeta.sin() + y * zeta.cos());
        
        (y.atan2(x) * r2d, z.asin() * r2d)
    }

    /// Compute Greenwich Mean Sidereal Time (GMST) in degrees.
    fn gmst(jd: f64) -> f64 {
        let t = (jd - 2451545.0) / 36525.0;
        let gmst = 280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t.powi(2) - (t.powi(3) / 38710000.0);
        gmst.rem_euclid(360.0)
    }

    fn utc_offset_accountance(utc_offset: i32, mut year: i32, mut month: i32, mut day: i32, mut hour: i32, mut minute: i32, mut second: i32) -> Result<(i32, i32, i32, i32, i32, i32), Box<dyn std::error::Error>> {
        // Add the utc offset to the hour variable, account for a negative or positive offset
        
        hour -= utc_offset;
        println!("utc time: {}", hour);

        let leap_year = year % 4 == 0;
        let months_31 = month == 1 || month == 3 || month == 5 || month == 7 || month == 8 || month == 10 || month == 12;
        let months_30 = month == 4 || month == 6 || month == 9 || month == 11;

        /* Loop continues until time is of correct format */
        while hour >= 24
            || day > 30 && months_30
            || day > 31 && months_31
            || day > 28 && !leap_year && month == 2 || day > 29 && leap_year && month == 2 
            || month > 12 || month <= 0
            || minute >= 60 || minute < 0
            || second >= 60 || second < 0
        {
            if second >= 60 { // Account for possible minute change
                minute += 1;
                second -= 60;
            }
            if second < 0 {
                minute -= 1;
                second += 59;
            }

            if minute >= 60 { // Account for possible hour change
                hour += 1;
                minute -= 60;
            }
            if minute < 0 {
                hour -= 1;
                minute += 59;
            }

            if hour <= -1 // Account for possible day change
            {
                day -= 1;
                hour = 24 + hour;
            }
            else if hour >= 24{
                day += 1;
                hour -= 24;
            }

            else if day <= 0  // Account for possible month change
            {
                month -= 1;
                if month == 2 // Account for february for leap year
                {
                    if !leap_year {
                        day += 28;
                    }
                    if leap_year {
                        day += 29;
                    }
                }
                else if months_30 {
                    day += 30;
                }
                else {
                    day += 31;
                }
            }
            else if day > 28 && !leap_year && month == 2 || day > 29 && leap_year && month == 2 { // Account for month change in feb
                month += 1;
                if leap_year {day -= 28} else {day -= 29}
            }
            else if day > 30 && months_30 {
                month += 1;
                day -= 30;
            }
            else if day > 31 && months_31 {
                month += 1;
                day -= 31;
            }  

            else if month >= 13 // Account for possible year change
            {
                year += 1;
                month -= 12;
            }
            else if month <= 0
            { 
                year -= 1;
                month += 12;
                day = 31;
            }
        }
        Ok((year, month, day, hour, minute, second))
    }
    }