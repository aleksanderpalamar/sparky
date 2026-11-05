use chrono::{Datelike, Days, Months, NaiveDate};
use fluent_bundle::FluentArgs;

#[derive(Debug, Clone)]
pub struct Calendar {
    shown: NaiveDate,
    today: NaiveDate,
}

#[derive(Debug, Clone, Copy)]
pub struct Day {
    pub date: NaiveDate,
    pub current: bool,
    pub today: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Previous,
    Next,
    Today,
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new(chrono::Local::now().date_naive())
    }
}

impl Calendar {
    pub fn new(today: NaiveDate) -> Self {
        Self {
            shown: today.with_day(1).unwrap_or(today),
            today,
        }
    }
    pub fn title(&self) -> String {
        let mut args = FluentArgs::new();
        args.set("month", super::clock::month(self.shown.month0()));
        args.set("year", self.shown.year().to_string());
        crate::i18n::message("calendar-title", Some(&args))
    }
    pub fn today(&self) -> NaiveDate {
        self.today
    }
    pub fn previous(&mut self) {
        self.shown = self
            .shown
            .checked_sub_months(Months::new(1))
            .unwrap_or(self.shown);
    }
    pub fn next(&mut self) {
        self.shown = self
            .shown
            .checked_add_months(Months::new(1))
            .unwrap_or(self.shown);
    }
    pub fn show_today(&mut self) {
        self.shown = self.today.with_day(1).unwrap_or(self.today);
    }
    pub fn update_today(&mut self, today: NaiveDate) {
        self.today = today;
    }
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Previous => self.previous(),
            Action::Next => self.next(),
            Action::Today => self.show_today(),
        }
    }

    pub fn days(&self) -> Vec<Day> {
        let offset = self.shown.weekday().num_days_from_monday() as u64;
        let first = self
            .shown
            .checked_sub_days(Days::new(offset))
            .unwrap_or(self.shown);
        (0..42)
            .filter_map(|index| {
                let date = first.checked_add_days(Days::new(index))?;
                Some(Day {
                    date,
                    current: date.month() == self.shown.month(),
                    today: date == self.today,
                })
            })
            .collect()
    }
}
