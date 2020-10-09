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

    let forecasts = match tenkijp::fetch_each_3hours_forecast() {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    for f in forecasts.iter().take(days) {
        let mut hour: Vec<String> = Vec::new();
        let mut kind: Vec<String> = Vec::new();
        let mut temperature: Vec<String> = Vec::new();
        let mut prob_precip: Vec<String> = Vec::new();
        let mut precipitation: Vec<String> = Vec::new();
        let mut humidity: Vec<String> = Vec::new();
        let mut wind_direction: Vec<String> = Vec::new();
        let mut wind_speed: Vec<String> = Vec::new();

        use tenkijp::weather::*;
        for (h, w) in &f.weathers {
            hour.push(format!("{:02}", h.hour()));
            match w {
                Announce::Past(w) | Announce::Regular(w) => {
                    kind.push(format!("{}", w.kind));
                    temperature.push(format!("{}", w.temperature));
                    prob_precip.push(
                        w.prob_precip
                            .map(|x| format!("{}", x))
                            .unwrap_or_else(|| "----".to_owned()),
                    );
                    precipitation.push(format!("{}", w.precipitation));
                    humidity.push(format!("{}", w.humidity));
                    wind_direction.push(format!("{}", w.wind_direction));
                    wind_speed.push(format!("{}", w.wind_speed));
                }
                NotYet => {
                    kind.push("----".to_owned());
                    temperature.push("----".to_owned());
                    prob_precip.push("----".to_owned());
                    precipitation.push("----".to_owned());
                    humidity.push("----".to_owned());
                    wind_direction.push("----".to_owned());
                    wind_speed.push("----".to_owned());
                }
            }
        }
        println!("=====================");
        println!("{:?}", hour);
        println!("{:?}", kind);
        println!("{:?}", temperature);
        println!("{:?}", prob_precip);
        println!("{:?}", precipitation);
        println!("{:?}", humidity);
        println!("{:?}", wind_direction);
        println!("{:?}", wind_speed);
    }

    use table::*;
    let mut columns = Vec::new();
    // upper left cell (empty)
    columns.push(Column {
        name: "".to_owned(),
        layout: ColumnLayout::Right,
    });
    let mut table = table::Table::<String>::empty(&forecasts[0].location, columns);
}
