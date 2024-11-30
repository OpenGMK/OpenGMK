use crate::{gml::Value, math::Real};
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
    time::macros::date!(1899 - 12 - 30).midnight()
}

fn now() -> time::OffsetDateTime {
    OffsetDateTime::now_utc()
        + time::Duration::seconds(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC).whole_seconds().into())
}

pub fn now_as_nanos() -> u128 {
    let datetime = now();
    datetime.unix_timestamp_nanos().try_into().unwrap_or(0)
}

fn i32_to_month(m: i32) -> Option<time::Month> {
    use time::Month::*;
    Some(match m {
        1 => January,
        2 => February,
        3 => March,
        4 => April,
        5 => May,
        6 => June,
        7 => July,
        8 => August,
        9 => September,
        10 => October,
        11 => November,
        12 => December,
        _ => return None,
    })
}

pub struct DateTime(PrimitiveDateTime);

impl DateTime {
    pub fn now() -> Self {
        let dt = now();
        Self(dt.date().with_time(dt.time()))
    }

    pub fn from_nanos(nanos: u128) -> Self {
        // manual unix timestamp because we need a PrimitiveDateTime
        // also split into seconds and nanos because time::Duration::nanoseconds takes an i64
        Self(time::macros::date!(1970 - 1 - 1).midnight()
            + time::Duration::new((nanos / 1_000_000_000) as _, (nanos % 1_000_000_000) as _))
    }

    pub fn date(&self) -> Self {
        Self(self.0.date().midnight())
    }

    pub fn time(&self) -> Self {
        Self(epoch().date().with_time(self.0.time()))
    }

    pub fn from_ymd(y: i32, m: i32, d: i32) -> Option<Self> {
        // GM doesn't support BCE so we won't either
        time::Date::from_calendar_date(y, i32_to_month(m)?, d as _).ok().map(|d| Self(d.midnight()))
    }

    pub fn from_hms(h: i32, m: i32, s: i32) -> Option<Self> {
        epoch().date().with_hms(h as _, m as _, s as _).ok().map(|dt| Self(dt))
    }

    pub fn from_ymdhms(y: i32, mo: i32, d: i32, h: i32, mi: i32, s: i32) -> Option<Self> {
        time::Date::from_calendar_date(y, i32_to_month(mo)?, d as _)
                .and_then(|d| d.with_hms(h as _, mi as _, s as _))
                .ok()
                .map(Self)
    }

    pub fn year(&self) -> i32 {
        self.0.date().year()
    }

    pub fn month(&self) -> u32 {
        (self.0.date().month() as u8).into()
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
        self.0.iso_week().into()
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
        let days = time::Duration::days(dt.trunc().to_i32().into());
        let ms = time::Duration::milliseconds((dt.fract() * Real::from(86400000)).floor().to_i32().into());
        // negate the time (see the inverse function for explanation)
        Self(epoch() + days + if dt > 0.into() { ms } else { -ms })
    }
}
