use chrono::{NaiveDate, NaiveTime};

const TIME: &str = "%I:%M %p";
const DATE: &str = "%A %-d %B";

pub fn time(moment: NaiveTime) -> String {
    moment.format(TIME).to_string()
}

pub fn date(day: NaiveDate) -> String {
    day.format(DATE).to_string()
}
