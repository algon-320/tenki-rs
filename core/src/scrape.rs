use crate::weather::*;
use chrono::prelude::*;
use itertools::izip;
use scraper::{Html, Selector};

#[derive(Debug)]
pub enum Error {
    InvalidLocation,
    NetworkError { msg: String },
    InvalidHtml { msg: String },
}
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLocation => write!(f, "Invalid Location"),
            Error::NetworkError { msg } => write!(f, "Network Error: {}", msg),
            Error::InvalidHtml { msg } => write!(f, "Invalid HTML: {}", msg),
        }
    }
}

async fn fetch_3days_forecast(location_code: &str, h: u8) -> Result<Box<[DailyForecast; 3]>> {
    assert!(h == 1 || h == 3);

    let url = reqwest::Url::parse(&format!(
        "https://tenki.jp/forecast/{}/{}.html",
        location_code,
        if h == 3 { "3hours" } else { "1hour" }
    ))
    .map_err(|_| Error::InvalidLocation)?;

    let response = reqwest::get(url.as_str())
        .await
        .map_err(|e| Error::NetworkError {
            msg: format!("{}", e),
        })?;
    if !response.status().is_success() {
        return Err(Error::InvalidLocation);
    }
    let html = response.text_with_charset("utf-8").await.unwrap();
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

    || -> std::result::Result<_, String> {
        let (location, announced_time) = {
            let mut text = document
                .select(&selector_location_announced_time)
                .next()
                .ok_or("location, announced_time not found")?
                .text();
            let location = text.next().ok_or("location not found")?;
            let announced_time = text.next().ok_or("announced_time not found")?;
            (location, announced_time)
        };

        let local_today = Local::today();
        let date_regex =
            regex::Regex::new(r#"(?:今日|明日|明後日)(?:&nbsp;|\W)((?:(\d+)年)?(\d+)月(\d+)日)?"#)
                .unwrap();

        let mut forecasts: Vec<DailyForecast> = Vec::new();
        for table in document.select(&selector_tables) {
            let table = Html::parse_fragment(&table.html());

            forecasts.push(DailyForecast {
                location: format!("{} ({})", location, announced_time),
                date: {
                    let date = table.select(&selector_head).next().unwrap().inner_html();
                    let grp = date_regex.captures(&date).ok_or("invalid date format")?;
                    if grp.get(1).is_some() {
                        let m: u32 = grp.get(3).unwrap().as_str().parse().unwrap();
                        let d: u32 = grp.get(4).unwrap().as_str().parse().unwrap();
                        let y: i32 = match grp.get(2) {
                            Some(sub) => sub.as_str().parse().unwrap(),
                            None => {
                                // check year wrapping
                                // NOTE: is this always correct?
                                if m == 1 && local_today.month() == 12 {
                                    local_today.year() + 1
                                } else {
                                    local_today.year()
                                }
                            }
                        };
                        chrono::NaiveDate::from_ymd(y, m, d)
                    } else {
                        forecasts.last().unwrap().date.succ()
                    }
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
                            fn collect_text(elem: scraper::ElementRef) -> String {
                                elem.text().collect::<String>().trim().to_owned()
                            }
                            fn parse<T>(s: &str, name: &str) -> std::result::Result<T, String>
                            where
                                T: std::str::FromStr,
                            {
                                s.parse()
                                    .map_err(|_| format!("Failed to parse {:?} as {}", s, name))
                            }

                            use selectors::attr::CaseSensitivity;
                            let past = hour
                                .value()
                                .has_class("past", CaseSensitivity::AsciiCaseInsensitive);
                            let not_yet = collect_text(kind).as_str() == "---";

                            let hour = {
                                let hour: u32 = parse(&collect_text(hour), "hour")?;
                                chrono::NaiveTime::from_hms(hour % 24, 0, 0)
                            };

                            if not_yet {
                                return Ok((hour, Announce::NotYet));
                            }

                            let weather = Weather {
                                kind: parse(&collect_text(kind), "kind")?,
                                temperature: parse(&collect_text(temp), "temp")?,
                                prob_precip: collect_text(prob_precip).parse().ok(),
                                precipitation: parse(&collect_text(precip), "precipitation")?,
                                humidity: parse(&collect_text(humid), "humidity")?,
                                wind_direction: parse(&collect_text(wind_dir), "wind_direction")?,
                                wind_speed: parse(&collect_text(wind_speed), "wind_speed")?,
                            };

                            if past {
                                Ok((hour, Announce::Past(weather)))
                            } else {
                                Ok((hour, Announce::Regular(weather)))
                            }
                        },
                    )
                    .collect::<std::result::Result<Vec<_>, String>>()?
                },
            });
        }

        use std::convert::TryInto;
        Ok(forecasts.into_boxed_slice().try_into().unwrap())
    }()
    .map_err(|e| Error::InvalidHtml { msg: e })
}

/// 3時間天気
pub async fn fetch_each_3hours_forecast(location_code: &str) -> Result<Box<[DailyForecast; 3]>> {
    fetch_3days_forecast(location_code, 3).await
}

/// 1時間天気
pub async fn fetch_each_1hour_forecast(location_code: &str) -> Result<Box<[DailyForecast; 3]>> {
    fetch_3days_forecast(location_code, 1).await
}

/// 10日間天気
pub async fn fetch_10days(_location_code: &str) -> Result<Box<[DailyForecast; 10]>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_3days() {
        let location_tsukuba = "3/11/4020/8220";
        match fetch_each_3hours_forecast(location_tsukuba).await {
            Err(Error::InvalidHtml { msg }) => {
                panic!("page layout updated? msg = {}", msg);
            }
            _ => {}
        }
        match fetch_each_1hour_forecast(location_tsukuba).await {
            Err(Error::InvalidHtml { msg }) => {
                panic!("page layout updated? msg = {}", msg);
            }
            _ => {}
        }
    }
}
