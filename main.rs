use reqwest::blocking::Client;
use std::error::Error;
use serde_json;
use std::fs;
use std::f64::consts::PI;


fn main() {
    let latitude = 47.359;
    let longitude = 121.986;
    let altitude = 26.7;
    let azimuth = 46.0;
    
    let year = 2020;
    let month = 10;
    let day = 26;
    let hour = 3;
    let minute = 43;
    let second = 30;

    let (ra, dec) = horizon_to_equatorial(
        latitude, longitude, altitude, azimuth, year, month, day, hour, minute, second,
    );

    let star = get_star(ra, dec).unwrap();

    println!("\nRA: {}\nDEC: {}\n\n{}", ra, dec, serde_json::to_string_pretty(&star).unwrap());
}

fn get_star(ra: f64, dec: f64) -> Result<serde_json::Value, Box<dyn Error>>
{
    let client = Client::new();
    let simbad_url = "http://simbad.u-strasbg.fr/simbad/sim-tap/sync";
    
    let query = format!(r#"
    SELECT TOP 1
        b.main_id AS star_id, 
        b.ra, 
        b.dec, 
        b.pmra, 
        b.pmdec, 
        b.plx_value AS parallax, 
        f.flux AS visual_magnitude, 
        b.sp_type AS spectral_type
    FROM 
        basic AS b
    JOIN 
        flux AS f ON b.oid = f.oidref
    WHERE 
        f.filter = 'V'
        AND f.flux <= 6.5
        AND CONTAINS(POINT('ICRS', b.ra, b.dec), CIRCLE('ICRS', {}, {}, 1.5)) = 1
    ORDER BY 
        visual_magnitude ASC
        "#, ra, dec);

    let params = [
        ("request", "doQuery"),
        ("lang", "ADQL"),
        ("format", "json"),
        ("query", &query),
    ];

    let response = client.post(simbad_url).form(&params).send()?;

    fs::write("./src/output.json", response.text()?)
        .expect("Unable to write to file");
    let file = fs::File::open("./src/output.json")
        .expect("File could not be opened");
    let json: serde_json::Value = serde_json::from_reader(file)
        .expect("File could not be read");

    Ok(json["data"].clone())

}

fn horizon_to_equatorial(
    latitude: f64,
    longitude: f64,
    altitude: f64,
    azimuth: f64,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
) -> (f64, f64) {
    let lat_rad = deg_to_rad(latitude);
    let alt_rad = deg_to_rad(altitude);
    let az_rad = deg_to_rad(azimuth);

    // Compute Julian Day and GMST
    let jd = julian_day(year, month, day, hour, minute, second);
    let gmst_deg = gmst(jd);

    // Compute Local Sidereal Time (LST)
    let lst_deg = (gmst_deg + longitude / 15.0).rem_euclid(360.0);
    let lst_rad = deg_to_rad(lst_deg);

    // Calculate declination (Dec)
    let sin_dec = alt_rad.sin() * lat_rad.sin() + alt_rad.cos() * lat_rad.cos() * az_rad.cos();
    let dec_rad = sin_dec.asin();

    // Compute Right Ascension (RA) from LST and Hour Angle (H)
    let mut h_rad = lst_rad - dec_rad;  // Use adjusted equation
    if h_rad < 0.0 {
        h_rad += 2.0 * PI;
    }

    // Convert RA and Dec from radians to degrees
    let ra_deg = rad_to_deg(h_rad).rem_euclid(360.0);
    let dec_deg = rad_to_deg(dec_rad);

    (ra_deg, dec_deg)
}

fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
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

/// Compute Greenwich Mean Sidereal Time (GMST) in degrees.
fn gmst(jd: f64) -> f64 {
    let d = jd - 2451545.0; // Days since J2000.0
    let t = d / 36525.0; // Julian centuries since J2000.0

    let gmst_deg = 280.46061837
        + 360.98564736629 * d
        + 0.000387933 * t.powi(2)
        - (t.powi(3) / 38710000.0);

    gmst_deg.rem_euclid(360.0) // Normalize to [0, 360)
}