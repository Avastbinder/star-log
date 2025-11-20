# Overview

A star finder program, written in Rust. 

![image](https://github.com/user-attachments/assets/6491eef1-9d10-4be5-a86f-c57372ed993d)


Enter in location, date, time, and horizontal coordinates of an observed star, and enter it into this program to find information about that star.

For linux, run /target/release/star-map
For Windows, run /target/x86_64-pc-windows-gnu/release/star-map.exe

[Software Demo Video](https://youtu.be/IfQDAYKjHxA)

# Cloud Database

Uses [WeatherAPI](https://www.weatherapi.com/), Googles timezone API to assist the user in inputting data, such as automatically converting to UTC, and getting location coordinates based on a zip code or city name.

Uses [SIMBAD's TAP Query](https://simbad.cds.unistra.fr/simbad/), which is used to locate the star and give information on the star like distance, brightness, star type.

# Development Environment

Used Visual Studio Code, and a very useful tool in the making of this application was [Stellarium](https://stellarium.org/), which allowed me to test the program, and also how accurate it is.

Built in Rust, and the libraries used in this project are;

- Iced, for the GUI
- Reqwests, to interface with the various API's used
- Serde_json, to manipulate the data from the API's
- Chrono, to make parsing date and time easier

# Useful Websites

- [Positional Astronomy: Conversion between horizontal and equatorial systems](https://sceweb.sce.uhcl.edu/helm/WEB-Positional%20Astronomy/Tutorial/Conversion/Conversion.html)
- [SIMBAD TAP Cheat sheet](https://simbad.cds.unistra.fr/simbad/tap/help/adqlHelp.html)

# Future Work

- Make more accurate (sometimes struggles with stars that are on cardinal directions)
- Save the previous searches into a file
- Return an image of the star
