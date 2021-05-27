use crate::{gml::Value, math::Real};
use std::convert::TryInto;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

/// Sleep for T minus 1 millisecond, and busywait for the rest of the duration.
pub fn sleep(dur: std::time::Duration) {
    // TODO: find a more precise way to sleep?
    let begin = std::time::Instant::now();
    if let Some(sleep_time) = dur.checked_sub(std::time::Duration::from_millis(1)) {
        std::thread::sleep(sleep_time);
    }
    while std::time::Instant::now() < begin + dur {}
}

fn epoch() -> PrimitiveDateTime {
    time::date!(1899 - 12 - 30).midnight()
}

fn now() -> time::OffsetDateTime {
    OffsetDateTime::now_utc()
        + time::Duration::seconds(UtcOffset::try_current_local_offset().unwrap_or(UtcOffset::UTC).as_seconds().into())
}

pub fn now_as_nanos() -> u128 {
    let datetime = now();
    datetime.unix_timestamp_nanos().try_into().unwrap_or(0)
}

pub struct DateTime(PrimitiveDateTime);

impl DateTime {
    pub fn now_or_nanos(nanos: Option<u128>) -> Self {
        Self(match nanos {
            Some(nanos) => {
                // manual unix timestamp because we need a PrimitiveDateTime
                // also split into seconds and nanos because time::Duration::nanoseconds takes an i64
                time::date!(1970 - 1 - 1).midnight()
                    + time::Duration::seconds((nanos / 1_000_000_000) as _)
                    + time::Duration::nanoseconds((nanos % 1_000_000_000) as _)
            },
            None => {
                let dt = now();
                dt.date().with_time(dt.time())
            },
        })
    }

    pub fn date(&self) -> Self {
        Self(self.0.date().midnight())
    }

    pub fn time(&self) -> Self {
        Self(epoch().date().with_time(self.0.time()))
    }

    pub fn from_ymd(y: i32, m: i32, d: i32) -> Option<Self> {
        // GM doesn't support BC so we won't either
        if y > 0 && m > 0 && d > 0 && m <= 12 && d <= 31 {
            time::Date::try_from_ymd(y, m as _, d as _).ok().map(|d| Self(d.midnight()))
        } else {
            None
        }
    }

    pub fn from_hms(h: i32, m: i32, s: i32) -> Option<Self> {
        if h >= 0 && m >= 0 && s >= 0 && h < 24 && m < 60 && s < 60 {
            epoch().date().try_with_hms(h as _, m as _, s as _).ok().map(|dt| Self(dt))
        } else {
            None
        }
    }

    pub fn from_ymdhms(y: i32, mo: i32, d: i32, h: i32, mi: i32, s: i32) -> Option<Self> {
        if y >= 0 && mo >= 0 && d >= 0 && h >= 0 && mi >= 0 && s >= 0 && mo <= 12 && d <= 31 && mi < 60 && s < 60 {
            time::Date::try_from_ymd(y, mo as _, d as _)
                .and_then(|d| d.try_with_hms(h as _, mi as _, s as _))
                .ok()
                .map(Self)
        } else {
            None
        }
    }

    pub fn year(&self) -> i32 {
        self.0.date().year()
    }

    pub fn month(&self) -> u32 {
        self.0.date().month().into()
    }

    pub fn day(&self) -> u32 {
        self.0.date().day().into()
    }

    pub fn day_of_year(&self) -> u32 {
        self.0.date().ordinal().into()
    }

    pub fn hour_of_year(&self) -> u32 {
        (self.day_of_year() - 1) * 24 + u32::from(self.0.time().hour())
    }

    pub fn minute_of_year(&self) -> u32 {
        self.hour_of_year() * 60 + u32::from(self.0.time().minute())
    }

    pub fn second_of_year(&self) -> u32 {
        self.minute_of_year() * 60 + u32::from(self.0.time().second())
    }

    pub fn hour(&self) -> u32 {
        self.0.time().hour().into()
    }

    pub fn minute(&self) -> u32 {
        self.0.time().minute().into()
    }

    pub fn second(&self) -> u32 {
        self.0.time().second().into()
    }

    pub fn week(&self) -> u32 {
        self.0.week().into()
    }

    pub fn weekday(&self) -> u32 {
        self.0.weekday().number_from_sunday().into()
    }
}

impl From<DateTime> for Real {
    fn from(dt: DateTime) -> Self {
        // calculate the ipart and fpart separately for maybe better precision?
        let ipart = Real::from((dt.0 - epoch()).whole_days() as f64);
        let fpart = Real::from((dt.time().0 - epoch()).whole_milliseconds() as f64) / Real::from(86400000);
        // the time part is the abs(fract()) of the datetime so that part increases backwards before the epoch
        if dt.0 >= epoch() { ipart + fpart } else { ipart - 1.into() - fpart }
    }
}

impl From<DateTime> for Value {
    fn from(dt: DateTime) -> Self {
        Real::from(dt).into()
    }
}

impl From<Real> for DateTime {
    fn from(dt: Real) -> Self {
        let days = time::Duration::days(dt.trunc().round().into());
        let ms = time::Duration::milliseconds((dt.fract() * Real::from(86400000)).floor().round().into());
        // negate the time (see the inverse function for explanation)
        Self(epoch() + days + if dt > 0.into() { ms } else { -ms })
    }
}
