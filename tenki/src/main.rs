use box_drawing_table::{
    ansi_term::{Color, Style},
    Align, Border, Cell, CellSize, Column, Row, Table,
};
use chrono::prelude::*;
use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, time::SystemTime};
use tenki_core::weather::DailyForecast;

fn japanese_weekday(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "月",
        Weekday::Tue => "火",
        Weekday::Wed => "水",
        Weekday::Thu => "木",
        Weekday::Fri => "金",
        Weekday::Sat => "土",
        Weekday::Sun => "日",
    }
}

fn get_weather_style_rgb(w: &tenki_core::weather::WeatherKind, past: bool) -> Style {
    use tenki_core::weather::WeatherKind::*;
    let mut base = Style::new().bold();
    if past {
        base = base.dimmed();
    }
    match w {
        Sunny => base.fg(Color::RGB(255, 159, 33)),
        Cloudy => base.fg(Color::RGB(194, 189, 182)),
        LittleRain => base.fg(Color::RGB(85, 208, 242)),
        WeakRain => base.fg(Color::RGB(85, 150, 242)),
        Rainy => base.fg(Color::RGB(0, 106, 255)),
        HeavyRain => base.fg(Color::RGB(143, 74, 255)),
        Storm => base.fg(Color::RGB(255, 18, 97)),
        DrySnow => base.fg(Color::RGB(64, 219, 154)),
        WetSnow => base.fg(Color::RGB(108, 224, 211)),
        Sleet => base.fg(Color::RGB(139, 180, 247)),
        Other(_) => base.fg(Color::RGB(255, 18, 180)),
    }
}

static CACHE_FILE_NAME: &str = "tenki.dump";

#[derive(Deserialize, Serialize)]
struct ForecastData {
    forecasts: Box<[DailyForecast; 3]>,
    location_code: String,
    fetched_date: SystemTime,
}

impl ForecastData {
    pub fn new(
        forecasts: Box<[DailyForecast; 3]>,
        location_code: String,
        fetched_date: SystemTime,
    ) -> Self {
        Self {
            forecasts,
            location_code,
            fetched_date,
        }
    }

    pub fn dump_to_file(&self, cache_file_name: &str) {
        let serialized_self = serde_json::to_string(self).expect("Serialize ForecastData");
        std::fs::File::create(cache_file_name)
            .and_then(|mut f| write!(f, "{}", serialized_self))
            .expect("Dump ForecastData to cache file");
    }

    pub fn fetch_forecast(location_code: &str, cache_file_name: &str) -> Self {
        ForecastData::read_from_valid_cache_file(location_code, cache_file_name).unwrap_or_else(
            || {
                let forecasts = match tenki_core::fetch_each_3hours_forecast(location_code) {
                    Ok(f) => f,
                    Err(e) => {
                        panic!("{}", e);
                    }
                };

                Self::new(forecasts, location_code.to_string(), SystemTime::now())
            },
        )
    }

    fn read_from_valid_cache_file(
        location_code: &str,
        cache_file_name: &str,
    ) -> Option<ForecastData> {
        use std::path::Path;
        use std::time::Duration;

        Path::new(cache_file_name)
            .exists()
            .then(|| ())
            .and_then(|_| File::open(cache_file_name).ok())
            .and_then(|f| serde_json::from_reader(f).ok())
            .filter(|fc: &ForecastData| {
                let duration = Duration::from_secs(60 * 60);
                SystemTime::now() - duration < fc.fetched_date && location_code == fc.location_code
            })
    }
}

fn main() {
    let app = App::new("tenki-rs")
        .author("algon-320 <algon.0320@mail.com>")
        .about("tenki.jp unofficial CLI client")
        .arg(Arg::with_name("days").required(false));
    let matches = app.get_matches();

    let days: usize = matches
        .value_of("days")
        .and_then(|days| match days.parse().ok() {
            Some(d) if (1..=3).contains(&d) => Some(d),
            _ => {
                eprintln!("tenki-rs: 'days' option must be an integer between 1 to 3",);
                None
            }
        })
        .unwrap_or(2);

    let tsukuba = "3/11/4020/8220"; // TODO: make if configuarable
    let forecast_data = ForecastData::fetch_forecast(tsukuba, CACHE_FILE_NAME);
    let forecasts = &forecast_data.forecasts;

    let title = forecasts[0].location.to_string();

    let mut columns = Vec::new();
    {
        columns.push(Column::VerticalBorder(Border::Double));
        columns.push(Column::Cells {
            width: CellSize::Flexible,
        });
        columns.push(Column::VerticalBorder(Border::Double));
        for _ in 0..forecasts[0].weathers.len() {
            columns.push(Column::Cells {
                width: CellSize::Fixed(6),
            });
            columns.push(Column::VerticalBorder(Border::Single));
        }
        columns.pop();
        columns.push(Column::VerticalBorder(Border::Double));
    }
    let mut table = Table::new(columns);

    for f in forecasts.iter().take(days) {
        use tenki_core::weather::*;

        let not_yet = Cell {
            value: "------".to_owned(),
            align: Align::Left,
            style: Style::default(),
        };

        table.append_row(Row::HorizontalBorder(Border::Double));
        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut hour = vec![Cell {
                    value: format!(
                        "{}月{}日({})",
                        f.date.month(),
                        f.date.day(),
                        japanese_weekday(f.date.weekday()),
                    ),
                    align: Align::Right,
                    style: Style::default(),
                }];
                hour.extend(f.weathers.iter().map(|(h, _)| Cell {
                    value: format!("{:02}", h.hour()),
                    align: Align::Left,
                    style: Style::new().bold(),
                }));
                hour
            },
        });

        table.append_row(Row::HorizontalBorder(Border::Single));

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut kind = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "天気".to_owned(),
                }];
                kind.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) => Cell {
                        value: w.kind.to_string(),
                        align: Align::Left,
                        style: get_weather_style_rgb(&w.kind, true),
                    },
                    Announce::Regular(w) => Cell {
                        value: w.kind.to_string(),
                        align: Align::Left,
                        style: get_weather_style_rgb(&w.kind, false),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                kind
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut temperature = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "気温(度)".to_owned(),
                }];
                temperature.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell {
                        value: w.temperature.to_string(),
                        align: Align::Left,
                        style: Style::default(),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                temperature
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut prob_precip = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "降水確率(%)".to_owned(),
                }];
                prob_precip.extend(f.weathers.iter().map(|(_, announce)| {
                    match announce {
                        Announce::Past(w) | Announce::Regular(w) => Cell {
                            value: w
                                .prob_precip
                                .map(|p| p.to_string())
                                .unwrap_or_else(|| "------".to_owned()),
                            align: Align::Left,
                            style: Style::default(),
                        },
                        Announce::NotYet => not_yet.clone(),
                    }
                }));
                prob_precip
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut precipitation = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "降水量(mm/h)".to_owned(),
                }];
                precipitation.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell {
                        value: w.precipitation.to_string(),
                        align: Align::Left,
                        style: Style::default(),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                precipitation
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut humidity = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "湿度(%)".to_owned(),
                }];
                humidity.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell {
                        value: w.humidity.to_string(),
                        align: Align::Left,
                        style: Style::default(),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                humidity
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut wind_direction = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "風向".to_owned(),
                }];
                wind_direction.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell {
                        value: w.wind_direction.to_string(),
                        align: Align::Left,
                        style: Style::default(),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                wind_direction
            },
        });

        table.append_row(Row::Cells {
            height: CellSize::Flexible,
            cells: {
                let mut wind_speed = vec![Cell {
                    style: Style::default(),
                    align: Align::Right,
                    value: "風速(m/s)".to_owned(),
                }];
                wind_speed.extend(f.weathers.iter().map(|(_, announce)| match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell {
                        value: w.wind_speed.to_string(),
                        align: Align::Left,
                        style: Style::default(),
                    },
                    Announce::NotYet => not_yet.clone(),
                }));
                wind_speed
            },
        });
    }
    table.append_row(Row::HorizontalBorder(Border::Double));

    println!("{}", title);
    print!("{}", table);

    forecast_data.dump_to_file(CACHE_FILE_NAME);
}
