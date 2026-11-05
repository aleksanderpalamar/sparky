use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use fluent_bundle::FluentArgs;

use crate::i18n;

const TIME: &str = "%I:%M %p";

pub fn time(moment: NaiveTime) -> String {
    moment.format(TIME).to_string()
}

pub fn date(day: NaiveDate) -> String {
    let mut args = FluentArgs::new();
    args.set("weekday", i18n::text(weekday(day.weekday())));
    args.set("day", day.day());
    args.set("month", i18n::text(month(day.month())));
    i18n::message("date", Some(&args))
}

fn weekday(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "weekday-monday",
        Weekday::Tue => "weekday-tuesday",
        Weekday::Wed => "weekday-wednesday",
        Weekday::Thu => "weekday-thursday",
        Weekday::Fri => "weekday-friday",
        Weekday::Sat => "weekday-saturday",
        Weekday::Sun => "weekday-sunday",
    }
}

fn month(number: u32) -> &'static str {
    const MONTHS: [&str; 12] = [
        "month-january",
        "month-february",
        "month-march",
        "month-april",
        "month-may",
        "month-june",
        "month-july",
        "month-august",
        "month-september",
        "month-october",
        "month-november",
        "month-december",
    ];
    number
        .checked_sub(1)
        .and_then(|index| MONTHS.get(index as usize))
        .copied()
        .unwrap_or("month-january")
}
