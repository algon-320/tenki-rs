use clap::{App, Arg, ArgMatches, SubCommand};

mod tenkijp;
mod weather;

const VERSION: &'static str = "0.1.0";
const APP_NAME: &'static str = "tenki-rs";

fn main() {
    let app = App::new(APP_NAME)
        .version(VERSION)
        .author("algon-320 <algon.0320@mail.com>")
        .about("tenki.jp unofficial CLI client")
        .arg(Arg::with_name("days").required(false));
    let matches = app.get_matches();
    let days: u8 = matches
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

    match tenkijp::fetch_each_3hours_forecast() {
        Ok(f) => {
            println!("{:?}", f);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
