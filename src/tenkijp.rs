use crate::weather::{Announce, DailyForecast, Weather, WeatherKind, WindDirection};
use chrono::prelude::*;
use itertools::izip;
use scraper::{Html, Selector};

#[derive(Debug)]
pub enum Error {
    NetworkError { msg: String },
    InvalidHtml,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NetworkError { msg } => write!(f, "Network Error: {}", msg),
            Error::InvalidHtml => write!(f, "Invalid HTML: "),
        }
    }
}

fn fetch_3days_forecast(h: u8) -> Result<Box<[DailyForecast; 3]>, Error> {
    assert!(h == 1 || h == 3);

    let url = format!(
        "https://tenki.jp/forecast/3/11/4020/8220/{}.html",
        if h == 3 { "3hours" } else { "1hour" }
    );
    let html = reqwest::blocking::get(url.as_str())
        .map_err(|e| Error::NetworkError {
            msg: format!("{}", e),
        })?
        .text_with_charset("utf-8")
        .unwrap();
    let document = Html::parse_document(&html);

    let selector_location_announced_time = Selector::parse("h2").unwrap();
    let selector_tables = Selector::parse(
        format!("#forecast-point-{}h-today, #forecast-point-{}h-tomorrow, #forecast-point-{}h-dayaftertomorrow", h, h, h).as_str()
    )
    .unwrap();
    let selector_head = Selector::parse("tr.head > td > div").unwrap();
    let selector_hour = Selector::parse("tr.hour > td > span").unwrap();
    let selector_kind = Selector::parse("tr.weather > td").unwrap();
    let selector_temperature = Selector::parse("tr.temperature > td").unwrap();
    let selector_prob_precip = Selector::parse("tr.prob-precip > td").unwrap();
    let selector_precipitation = Selector::parse("tr.precipitation > td").unwrap();
    let selector_humidity = Selector::parse("tr.humidity > td").unwrap();
    let selector_wind_dir = Selector::parse("tr.wind-direction > td, tr.wind-blow > td").unwrap();
    let selector_wind_speed = Selector::parse("tr.wind-speed > td").unwrap();

    move || -> Option<Box<[DailyForecast; 3]>> {
        let (location, announced_time) = {
            let mut text = document
                .select(&selector_location_announced_time)
                .next()?
                .text();
            let location = text.next()?;
            let announced_time = text.next()?;
            (location, announced_time)
        };

        let local_today = chrono::Local::today();
        let date_regex = regex::Regex::new(r#"(\d+)月(\d+)日"#).unwrap();
        let parse_date = |input: &str| -> Option<chrono::NaiveDate> {
            let grp = date_regex.captures(input)?;
            let m: u32 = grp.get(1)?.as_str().parse().unwrap();
            let d: u32 = grp.get(2)?.as_str().parse().unwrap();
            // check year wrapping
            // NOTE: is this always correct?
            let y: i32 = if m == 1 && local_today.month() == 12 {
                local_today.year() + 1
            } else {
                local_today.year()
            };
            Some(chrono::NaiveDate::from_ymd(y, m, d))
        };

        let mut forecasts = Vec::new();
        for table in document.select(&selector_tables) {
            let table = Html::parse_fragment(&table.html());
            forecasts.push(DailyForecast {
                location: format!("{} ({})", location, announced_time),
                date: {
                    let date = table.select(&selector_head).next().unwrap().inner_html();
                    parse_date(date.as_str())?
                },
                weathers: {
                    izip!(
                        table.select(&selector_hour),
                        table.select(&selector_kind),
                        table.select(&selector_temperature),
                        table.select(&selector_prob_precip),
                        table.select(&selector_precipitation),
                        table.select(&selector_humidity),
                        table.select(&selector_wind_dir),
                        table.select(&selector_wind_speed),
                    )
                    .map(
                        |(hour, kind, temp, prob_precip, precip, humid, wind_dir, wind_speed)| {
                            let collect_text = |elem: scraper::ElementRef| {
                                elem.text().collect::<String>().trim().to_owned()
                            };

                            use selectors::attr::CaseSensitivity;
                            let past = hour
                                .value()
                                .has_class("past", CaseSensitivity::AsciiCaseInsensitive);

                            let hour = {
                                let hour: u32 = collect_text(hour).parse().ok()?;
                                chrono::NaiveTime::from_hms(hour % 24, 0, 0)
                            };

                            let kind = collect_text(kind);
                            if kind.as_str() == "---" {
                                Some((hour, Announce::NotYet))
                            } else {
                                let weather = Weather {
                                    kind: WeatherKind::parse(kind.as_str()),
                                    temperature: collect_text(temp).parse().ok()?,
                                    prob_precip: collect_text(prob_precip).parse().ok(),
                                    precipitation: collect_text(precip).parse().ok()?,
                                    humidity: collect_text(humid).parse().ok()?,
                                    wind_direction: WindDirection::parse(
                                        collect_text(wind_dir).as_str(),
                                    )?,
                                    wind_speed: collect_text(wind_speed).parse().ok()?,
                                };

                                if past {
                                    Some((hour, Announce::Past(weather)))
                                } else {
                                    Some((hour, Announce::Regular(weather)))
                                }
                            }
                        },
                    )
                    .collect::<Option<Vec<_>>>()?
                },
            });
        }

        use std::convert::TryInto;
        Some(forecasts.into_boxed_slice().try_into().unwrap())
    }()
    .ok_or(Error::InvalidHtml)
}

/// 3時間天気
#[allow(dead_code)]
pub fn fetch_each_3hours_forecast() -> Result<Box<[DailyForecast; 3]>, Error> {
    fetch_3days_forecast(3)
}

/// 1時間天気
#[allow(dead_code)]
pub fn fetch_each_1hour_forecast() -> Result<Box<[DailyForecast; 3]>, Error> {
    fetch_3days_forecast(1)
}

/// 10日間天気
#[allow(dead_code)]
pub fn fetch_10days() -> Result<Box<[DailyForecast; 10]>, Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_3days() {
        match fetch_each_3hours_forecast() {
            Err(Error::InvalidHtml) => {
                panic!("page layout updated?");
            }
            _ => {}
        }
        match fetch_each_1hour_forecast() {
            Err(Error::InvalidHtml) => {
                panic!("page layout updated?");
            }
            _ => {}
        }
    }
}
