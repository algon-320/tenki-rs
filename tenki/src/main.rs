use chrono::prelude::*;
use clap::{App, Arg, ArgMatches, SubCommand};

const VERSION: &'static str = "0.1.0";
const APP_NAME: &'static str = "tenki-rs";

mod table;

fn main() {
    let app = App::new(APP_NAME)
        .version(VERSION)
        .author("algon-320 <algon.0320@mail.com>")
        .about("tenki.jp unofficial CLI client")
        .arg(Arg::with_name("days").required(false));
    let matches = app.get_matches();
    let days: usize = matches
        .value_of("days")
        .and_then(|days| match days.parse().ok() {
            Some(d) if 1 <= d && d <= 3 => Some(d),
            _ => {
                println!(
                    "{}: 'days' option must be an integer between 1 to 3",
                    APP_NAME
                );
                None
            }
        })
        .unwrap_or(2);
    dbg!(days);

    let forecasts = match tenki_core::fetch_each_3hours_forecast() {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let mut table = table::Table::empty(&forecasts[0].location, 4 + forecasts[0].weathers.len());
    for f in forecasts.iter().take(days) {
        let mut hour = Vec::new();
        const FIELDS: usize = 7;
        use table::Cell;
        let mut rows = vec![
            vec![
                Cell::VarticalBorder,
                Cell::new_right(format!("{}月{}日", f.date.month(), f.date.day())),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("気温(度)"),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("降水確率(%)"),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("降水量(mm/h)"),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("湿度(%)"),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("風向"),
                Cell::VarticalBorder,
            ],
            vec![
                Cell::VarticalBorder,
                Cell::new_right("風速(m/s)"),
                Cell::VarticalBorder,
            ],
        ];

        use tenki_core::weather::*;
        for (h, w) in &f.weathers {
            hour.push(format!("{:02}", h.hour()));

            let fields: [Option<&dyn std::fmt::Display>; FIELDS] = match w {
                Announce::Past(w) | Announce::Regular(w) => [
                    Some(&w.kind),
                    Some(&w.temperature),
                    w.prob_precip.as_ref().map(|p| p as &dyn std::fmt::Display),
                    Some(&w.precipitation),
                    Some(&w.humidity),
                    Some(&w.wind_direction),
                    Some(&w.wind_speed),
                ],
                Announce::NotYet => [None; FIELDS],
            };
            for (row, field) in rows.iter_mut().zip(&fields) {
                row.push(Cell::new_right(
                    field
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "----".to_owned()),
                ));
            }
        }
        for row in rows.iter_mut() {
            row.push(Cell::VarticalBorder);
        }

        let mut row = Vec::new();
        row.push(Cell::VarticalBorder);
        row.push(Cell::Empty);
        row.push(Cell::VarticalBorder);
        for h in hour {
            row.push(Cell::new_right(h.to_string()));
        }
        row.push(Cell::VarticalBorder);

        table.add_horizontal_border();

        for row in rows {
            table.add_row(row).unwrap();
        }
    }
    table.add_horizontal_border();
    println!("{}", table);
}
