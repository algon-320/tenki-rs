use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Display, EnumString)]
pub enum WeatherKind {
    #[strum(to_string = "晴れ")]
    Sunny,
    #[strum(to_string = "曇り")]
    Cloudy,
    #[strum(to_string = "小雨")]
    LittleRain,
    #[strum(to_string = "弱雨")]
    WeakRain,
    #[strum(to_string = "雨")]
    Rainy,
    #[strum(to_string = "強雨")]
    HeavyRain,
    #[strum(to_string = "豪雨")]
    Storm,
    #[strum(to_string = "乾雪")]
    DrySnow,
    #[strum(to_string = "湿雪")]
    WetSnow,
    #[strum(to_string = "みぞれ")]
    Sleet,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Display, EnumString)]
pub enum WindDirection {
    #[strum(to_string = "北")]
    N,
    #[strum(to_string = "北北東")]
    NNE,
    #[strum(to_string = "北東")]
    NE,
    #[strum(to_string = "東北東")]
    ENE,
    #[strum(to_string = "東")]
    E,
    #[strum(to_string = "東南東")]
    ESE,
    #[strum(to_string = "南東")]
    SE,
    #[strum(to_string = "南南東")]
    SSE,
    #[strum(to_string = "南")]
    S,
    #[strum(to_string = "南南西")]
    SSW,
    #[strum(to_string = "南西")]
    SW,
    #[strum(to_string = "西南西")]
    WSW,
    #[strum(to_string = "西")]
    W,
    #[strum(to_string = "西北西")]
    WNW,
    #[strum(to_string = "北西")]
    NW,
    #[strum(to_string = "北北西")]
    NNW,

    #[strum(to_string = "静穏")]
    Calm,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weather {
    pub kind: WeatherKind,             //
    pub temperature: f32,              // ℃
    pub prob_precip: Option<u8>,       // %
    pub precipitation: u32,            // mm/h
    pub humidity: u32,                 // %
    pub wind_direction: WindDirection, //
    pub wind_speed: u32,               // m
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Announce {
    Past(Weather),
    Regular(Weather),
    NotYet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DailyForecast {
    pub location: String,
    pub date: NaiveDate,
    pub weathers: Vec<(NaiveTime, Announce)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::string::ToString;

    #[test]
    fn test_parse_weather_kind() {
        let w = WeatherKind::from_str("晴れ").unwrap();
        let e = WeatherKind::Sunny;
        assert_eq!(w, e);
        let w = WeatherKind::from_str("曇り").unwrap();
        let e = WeatherKind::Cloudy;
        assert_eq!(w, e);
        let w = WeatherKind::from_str("雨").unwrap();
        let e = WeatherKind::Rainy;
        assert_eq!(w, e);
        let w = WeatherKind::from_str("弱雨").unwrap();
        let e = WeatherKind::WeakRain;
        assert_eq!(w, e);

        let w = WeatherKind::from_str("ひょう").unwrap();
        let e = WeatherKind::Other("ひょう".to_owned());
        assert_eq!(w, e);
    }

    #[test]
    fn test_display_weather_kind() {
        assert_eq!(WeatherKind::Sunny.to_string(), "晴れ".to_owned());
    }

    #[test]
    fn test_parse_dir() {
        let d = WindDirection::from_str("北").unwrap();
        let e = WindDirection::N;
        assert_eq!(d, e);
        let d = WindDirection::from_str("北北西").unwrap();
        let e = WindDirection::NNW;
        assert_eq!(d, e);
    }

    #[test]
    fn test_display_dir() {
        assert_eq!(WindDirection::N.to_string(), "北".to_owned());
    }
}
