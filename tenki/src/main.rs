use box_drawing_table::{
    ansi_term::{Color, Style},
    Align, Border, Cell, Column, Row, Table,
};
use chrono::prelude::*;
use clap::Parser;
use std::io::Write as _;

/// unofficial CLI client for tenki.jp
#[derive(Parser, Debug)]
#[command(author, version)]
struct TenkiOptions {
    /// the number of days to display (integer between 1 to 3)
    #[arg(short, long, default_value = "2")]
    days: usize,

    /// location to display (by default, Tsukuba City)
    #[arg(default_value = "3/11/4020/8220")]
    location: String,
}

#[tokio::main]
async fn main() {
    let options = TenkiOptions::parse();

    // NOTE: clippy maybe supports this kind of verification?
    if !(1 <= options.days && options.days <= 3) {
        eprintln!("tenki: 'days' option must be an integer between 1 to 3");
        return;
    }

    let loading_indicator = tokio::task::spawn(async {
        print!("loading ");
        std::io::stdout().flush().unwrap();
        loop {
            print!(".");
            std::io::stdout().flush().unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    });

    let forecasts = match tenki_core::fetch_each_3hours_forecast(&options.location).await {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    loading_indicator.abort();
    print!("\r\x1b[J"); // FIXME
    std::io::stdout().flush().unwrap();

    let title = forecasts[0].location.to_string();

    let mut columns = Vec::new();
    {
        columns.push(Border::Double.into());
        columns.push(Column::flexible_width());
        columns.push(Border::Double.into());
        for _ in 0..forecasts[0].weathers.len() {
            columns.push(Column::fixed_width(6));
            columns.push(Border::Single.into());
        }
        columns.pop();
        columns.push(Border::Double.into());
    }
    let mut table = Table::new(columns);

    for f in forecasts.iter().take(options.days) {
        use tenki_core::weather::*;

        let not_yet = Cell {
            value: "------".to_owned(),
            align: Align::Left,
            style: Style::default(),
        };

        table.append_row(Border::Double.into());
        table.append_row(Row::flexible_height({
            let date = format!(
                "{}月{}日({})",
                f.date.month(),
                f.date.day(),
                japanese_weekday(f.date.weekday()),
            );
            let mut hour = vec![Cell::right(date)];
            hour.extend(f.weathers.iter().map(|(h, _)| {
                Cell::left_with_style(format!("{:02}時", h.hour()), Style::new().bold())
            }));
            hour
        }));

        table.append_row(Border::Single.into());

        table.append_row(Row::flexible_height({
            let mut kind = vec![Cell::right("天気")];
            kind.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) => {
                    Cell::left_with_style(&w.kind, get_weather_style_rgb(&w.kind, true))
                }
                Announce::Regular(w) => {
                    Cell::left_with_style(&w.kind, get_weather_style_rgb(&w.kind, false))
                }
                Announce::NotYet => not_yet.clone(),
            }));
            kind
        }));

        table.append_row(Row::flexible_height({
            let mut temperature = vec![Cell::right("気温(度)")];
            temperature.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) | Announce::Regular(w) => Cell::left(w.temperature),
                Announce::NotYet => not_yet.clone(),
            }));
            temperature
        }));

        table.append_row(Row::flexible_height({
            let mut prob_precip = vec![Cell::right("降水確率(%)")];
            prob_precip.extend(f.weathers.iter().map(|(_, announce)| {
                match announce {
                    Announce::Past(w) | Announce::Regular(w) => Cell::left(
                        w.prob_precip
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "------".to_owned()),
                    ),
                    Announce::NotYet => not_yet.clone(),
                }
            }));
            prob_precip
        }));

        table.append_row(Row::flexible_height({
            let mut precipitation = vec![Cell::right("降水量(mm/h)")];
            precipitation.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) | Announce::Regular(w) => Cell::left(w.precipitation),
                Announce::NotYet => not_yet.clone(),
            }));
            precipitation
        }));

        table.append_row(Row::flexible_height({
            let mut humidity = vec![Cell::right("湿度(%)")];
            humidity.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) | Announce::Regular(w) => Cell::left(w.humidity),
                Announce::NotYet => not_yet.clone(),
            }));
            humidity
        }));

        table.append_row(Row::flexible_height({
            let mut wind_direction = vec![Cell::right("風向")];
            wind_direction.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) | Announce::Regular(w) => Cell::left(w.wind_direction),
                Announce::NotYet => not_yet.clone(),
            }));
            wind_direction
        }));

        table.append_row(Row::flexible_height({
            let mut wind_speed = vec![Cell::right("風速(m/s)")];
            wind_speed.extend(f.weathers.iter().map(|(_, announce)| match announce {
                Announce::Past(w) | Announce::Regular(w) => Cell::left(w.wind_speed),
                Announce::NotYet => not_yet.clone(),
            }));
            wind_speed
        }));
    }
    table.append_row(Border::Double.into());

    println!("{}", title);
    print!("{}", table);
}

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
