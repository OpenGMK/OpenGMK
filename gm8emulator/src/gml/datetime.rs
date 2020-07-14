use crate::{gml::Value, math::Real};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};

fn epoch() -> NaiveDateTime {
    NaiveDate::from_ymd(1899, 12, 30).and_hms(0, 0, 0)
}

pub struct DateTime(NaiveDateTime);

impl DateTime {
    pub fn now_or_nanos(nanos: Option<u128>) -> Self {
        if let Some(nanos) = nanos {
            Self(NaiveDateTime::from_timestamp((nanos / 1_000_000_000) as i64, (nanos % 1_000_000_000) as u32))
        } else {
            Self(chrono::Local::now().naive_local())
        }
    }

    pub fn date(&self) -> Self {
        Self(self.0.date().and_hms(0, 0, 0))
    }

    pub fn time(&self) -> Self {
        Self(epoch().date().and_time(self.0.time()))
    }

    pub fn from_ymd(y: i32, m: i32, d: i32) -> Option<Self> {
        // GM doesn't support BC so we won't either
        if y > 0 && m > 0 && d > 0 {
            NaiveDate::from_ymd_opt(y, m as u32, d as u32).map(|d| Self(d.and_hms(0, 0, 0)))
        } else {
            None
        }
    }

    pub fn from_hms(h: i32, m: i32, s: i32) -> Option<Self> {
        if h > 0 && m > 0 && s > 0 {
            epoch().date().and_hms_opt(h as u32, m as u32, s as u32).map(|dt| Self(dt))
        } else {
            None
        }
    }

    pub fn year(&self) -> i32 {
        self.0.date().year()
    }

    pub fn month(&self) -> u32 {
        self.0.date().month()
    }

    pub fn day(&self) -> u32 {
        self.0.date().day()
    }

    pub fn hour(&self) -> u32 {
        self.0.time().hour()
    }

    pub fn minute(&self) -> u32 {
        self.0.time().minute()
    }

    pub fn second(&self) -> u32 {
        self.0.time().second()
    }

    pub fn week(&self) -> u32 {
        self.0.iso_week().week()
    }

    pub fn weekday(&self) -> u32 {
        self.0.weekday().number_from_sunday()
    }
}

impl From<DateTime> for Real {
    fn from(dt: DateTime) -> Self {
        // calculate the ipart and fpart separately for maybe better precision?
        let ipart = Real::from((dt.0 - epoch()).num_days() as f64);
        let fpart = Real::from((dt.time().0 - epoch()).num_milliseconds() as f64) / Real::from(86400000);
        ipart + fpart
    }
}

impl From<DateTime> for Value {
    fn from(dt: DateTime) -> Self {
        Real::from(dt).into()
    }
}

impl From<Real> for DateTime {
    fn from(dt: Real) -> Self {
        let days = Duration::days(dt.floor().round().into());
        let ms = Duration::milliseconds((dt.fract() * Real::from(86400000)).floor().round().into());
        Self(epoch() + days + ms)
    }
}
