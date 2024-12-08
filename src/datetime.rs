use core::sync::atomic::AtomicUsize;
use spin::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct DateTime {
    pub day: u8,
    pub month: u8,
    pub year: u16,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

pub static TICKS: AtomicUsize = AtomicUsize::new(0);
pub static CURRENT_TIME: Mutex<DateTime> = Mutex::new(DateTime {
    day: 1,
    month: 1,
    year: 2023,
    hours: 12,
    minutes: 0,
    seconds: 0,
});

impl DateTime {
    pub fn update(&mut self) {
        self.seconds += 1;
        if self.seconds >= 60 {
            self.seconds = 0;
            self.minutes += 1;
            if self.minutes >= 60 {
                self.minutes = 0;
                self.hours += 1;
                if self.hours >= 24 {
                    self.hours = 0;
                    self.day += 1;
                    if self.day > days_in_month(self.month, self.year) {
                        self.day = 1;
                        self.month += 1;
                        if self.month > 12 {
                            self.month = 1;
                            self.year += 1;
                        }
                    }
                }
            }
        }
    }
}

fn days_in_month(month: u8, year: u16) -> u8 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn get_time() -> (u8, u8, u8) {
    let time = CURRENT_TIME.lock();
    (time.hours, time.minutes, time.seconds)
}

pub fn get_date() -> (u8, u8, u16) {
    let time = CURRENT_TIME.lock();
    (time.day, time.month, time.year)
}

pub fn set_time(hours: u8, minutes: u8, seconds: u8) {
    let mut time = CURRENT_TIME.lock();
    time.hours = hours;
    time.minutes = minutes;
    time.seconds = seconds;
}

pub fn set_date(day: u8, month: u8, year: u16) {
    let mut time = CURRENT_TIME.lock();
    time.day = day;
    time.month = month;
    time.year = year;
}
