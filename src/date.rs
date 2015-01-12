// This is a part of rust-chrono.
// Copyright (c) 2014-2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * ISO 8601 calendar date with timezone.
 */

use std::{fmt, hash};
use std::cmp::Ordering;
use std::ops::{Add, Sub};

use {Weekday, Datelike};
use duration::Duration;
use offset::{Offset, OffsetState};
use offset::utc::UTC;
use naive;
use naive::date::NaiveDate;
use naive::time::NaiveTime;
use datetime::DateTime;
use format::DelayedFormat;

/// ISO 8601 calendar date with timezone.
#[derive(Clone)]
pub struct Date<Off: Offset> {
    date: NaiveDate,
    offset: Off::State,
}

/// The minimum possible `Date`.
pub const MIN: Date<UTC> = Date { date: naive::date::MIN, offset: UTC };
/// The maximum possible `Date`.
pub const MAX: Date<UTC> = Date { date: naive::date::MAX, offset: UTC };

impl<Off: Offset> Date<Off> {
    /// Makes a new `Date` with given *UTC* date and offset.
    /// The local date should be constructed via the `Offset` trait.
    //
    // note: this constructor is purposedly not named to `new` to discourage the direct usage.
    #[inline]
    pub fn from_utc(date: NaiveDate, offset: Off::State) -> Date<Off> {
        Date { date: date, offset: offset }
    }

    /// Makes a new `DateTime` from the current date and given `NaiveTime`.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid datetime.
    #[inline]
    pub fn and_time(&self, time: NaiveTime) -> Option<DateTime<Off>> {
        let localdt = self.naive_local().and_time(time);
        self.timezone().from_local_datetime(&localdt).single()
    }

    /// Makes a new `DateTime` from the current date, hour, minute and second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute and/or second.
    #[inline]
    pub fn and_hms(&self, hour: u32, min: u32, sec: u32) -> DateTime<Off> {
        self.and_hms_opt(hour, min, sec).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute and second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute and/or second.
    #[inline]
    pub fn and_hms_opt(&self, hour: u32, min: u32, sec: u32) -> Option<DateTime<Off>> {
        NaiveTime::from_hms_opt(hour, min, sec).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and millisecond.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or millisecond.
    #[inline]
    pub fn and_hms_milli(&self, hour: u32, min: u32, sec: u32, milli: u32) -> DateTime<Off> {
        self.and_hms_milli_opt(hour, min, sec, milli).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and millisecond.
    /// The millisecond part can exceed 1,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or millisecond.
    #[inline]
    pub fn and_hms_milli_opt(&self, hour: u32, min: u32, sec: u32,
                             milli: u32) -> Option<DateTime<Off>> {
        NaiveTime::from_hms_milli_opt(hour, min, sec, milli).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and microsecond.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or microsecond.
    #[inline]
    pub fn and_hms_micro(&self, hour: u32, min: u32, sec: u32, micro: u32) -> DateTime<Off> {
        self.and_hms_micro_opt(hour, min, sec, micro).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and microsecond.
    /// The microsecond part can exceed 1,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or microsecond.
    #[inline]
    pub fn and_hms_micro_opt(&self, hour: u32, min: u32, sec: u32,
                             micro: u32) -> Option<DateTime<Off>> {
        NaiveTime::from_hms_micro_opt(hour, min, sec, micro).and_then(|time| self.and_time(time))
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and nanosecond.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Fails on invalid hour, minute, second and/or nanosecond.
    #[inline]
    pub fn and_hms_nano(&self, hour: u32, min: u32, sec: u32, nano: u32) -> DateTime<Off> {
        self.and_hms_nano_opt(hour, min, sec, nano).expect("invalid time")
    }

    /// Makes a new `DateTime` from the current date, hour, minute, second and nanosecond.
    /// The nanosecond part can exceed 1,000,000,000 in order to represent the leap second.
    /// The offset in the current date is preserved.
    ///
    /// Returns `None` on invalid hour, minute, second and/or nanosecond.
    #[inline]
    pub fn and_hms_nano_opt(&self, hour: u32, min: u32, sec: u32,
                            nano: u32) -> Option<DateTime<Off>> {
        NaiveTime::from_hms_nano_opt(hour, min, sec, nano).and_then(|time| self.and_time(time))
    }

    /// Makes a new `Date` for the next date.
    ///
    /// Fails when `self` is the last representable date.
    #[inline]
    pub fn succ(&self) -> Date<Off> {
        self.succ_opt().expect("out of bound")
    }

    /// Makes a new `Date` for the next date.
    ///
    /// Returns `None` when `self` is the last representable date.
    #[inline]
    pub fn succ_opt(&self) -> Option<Date<Off>> {
        self.date.succ_opt().map(|date| Date::from_utc(date, self.offset.clone()))
    }

    /// Makes a new `Date` for the prior date.
    ///
    /// Fails when `self` is the first representable date.
    #[inline]
    pub fn pred(&self) -> Date<Off> {
        self.pred_opt().expect("out of bound")
    }

    /// Makes a new `Date` for the prior date.
    ///
    /// Returns `None` when `self` is the first representable date.
    #[inline]
    pub fn pred_opt(&self) -> Option<Date<Off>> {
        self.date.pred_opt().map(|date| Date::from_utc(date, self.offset.clone()))
    }

    /// Retrieves an associated offset state.
    #[inline]
    pub fn offset<'a>(&'a self) -> &'a Off::State {
        &self.offset
    }

    /// Retrieves an associated offset.
    #[inline]
    pub fn timezone(&self) -> Off {
        Offset::from_state(&self.offset)
    }

    /// Changes the associated offset.
    /// This does not change the actual `Date` (but will change the string representation).
    #[inline]
    pub fn with_timezone<Off2: Offset>(&self, tz: &Off2) -> Date<Off2> {
        tz.from_utc_date(&self.date)
    }

    /// Returns a view to the naive UTC date.
    #[inline]
    pub fn naive_utc(&self) -> NaiveDate {
        self.date
    }

    /// Returns a view to the naive local date.
    #[inline]
    pub fn naive_local(&self) -> NaiveDate {
        self.date + self.offset.local_minus_utc()
    }
}

/// Maps the local date to other date with given conversion function.
fn map_local<Off: Offset, F>(d: &Date<Off>, mut f: F) -> Option<Date<Off>>
        where F: FnMut(NaiveDate) -> Option<NaiveDate> {
    f(d.naive_local()).and_then(|date| d.timezone().from_local_date(&date).single())
}

impl<Off: Offset> Date<Off> where Off::State: fmt::Display {
    /// Formats the date in the specified format string.
    /// See the `format` module on the supported escape sequences.
    #[inline]
    pub fn format<'a>(&'a self, fmt: &'a str) -> DelayedFormat<'a> {
        DelayedFormat::new_with_offset(Some(self.naive_local()), None, &self.offset, fmt)
    }
}

impl<Off: Offset> Datelike for Date<Off> {
    #[inline] fn year(&self) -> i32 { self.naive_local().year() }
    #[inline] fn month(&self) -> u32 { self.naive_local().month() }
    #[inline] fn month0(&self) -> u32 { self.naive_local().month0() }
    #[inline] fn day(&self) -> u32 { self.naive_local().day() }
    #[inline] fn day0(&self) -> u32 { self.naive_local().day0() }
    #[inline] fn ordinal(&self) -> u32 { self.naive_local().ordinal() }
    #[inline] fn ordinal0(&self) -> u32 { self.naive_local().ordinal0() }
    #[inline] fn weekday(&self) -> Weekday { self.naive_local().weekday() }
    #[inline] fn isoweekdate(&self) -> (i32, u32, Weekday) { self.naive_local().isoweekdate() }

    #[inline]
    fn with_year(&self, year: i32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_year(year))
    }

    #[inline]
    fn with_month(&self, month: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_month(month))
    }

    #[inline]
    fn with_month0(&self, month0: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_month0(month0))
    }

    #[inline]
    fn with_day(&self, day: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_day(day))
    }

    #[inline]
    fn with_day0(&self, day0: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_day0(day0))
    }

    #[inline]
    fn with_ordinal(&self, ordinal: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_ordinal(ordinal))
    }

    #[inline]
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Date<Off>> {
        map_local(self, |date| date.with_ordinal0(ordinal0))
    }
}

impl<Off: Offset, Off2: Offset> PartialEq<Date<Off2>> for Date<Off> {
    fn eq(&self, other: &Date<Off2>) -> bool { self.date == other.date }
}

impl<Off: Offset> Eq for Date<Off> {
}

impl<Off: Offset> PartialOrd for Date<Off> {
    fn partial_cmp(&self, other: &Date<Off>) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

impl<Off: Offset> Ord for Date<Off> {
    fn cmp(&self, other: &Date<Off>) -> Ordering { self.date.cmp(&other.date) }
}

impl<Off: Offset, H: hash::Hasher + hash::Writer> hash::Hash<H> for Date<Off> {
    fn hash(&self, state: &mut H) { self.date.hash(state) }
}

impl<Off: Offset> Add<Duration> for Date<Off> {
    type Output = Date<Off>;

    fn add(self, rhs: Duration) -> Date<Off> {
        Date { date: self.date + rhs, offset: self.offset }
    }
}

impl<Off: Offset, Off2: Offset> Sub<Date<Off2>> for Date<Off> {
    type Output = Duration;

    fn sub(self, rhs: Date<Off2>) -> Duration { self.date - rhs.date }
}

impl<Off: Offset> Sub<Duration> for Date<Off> {
    type Output = Date<Off>;

    #[inline]
    fn sub(self, rhs: Duration) -> Date<Off> { self.add(-rhs) }
}

impl<Off: Offset> fmt::Debug for Date<Off> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.naive_local(), self.offset)
    }
}

impl<Off: Offset> fmt::Display for Date<Off> where Off::State: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.naive_local(), self.offset)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use duration::Duration;
    use naive::date::NaiveDate;
    use naive::time::NaiveTime;
    use naive::datetime::NaiveDateTime;
    use offset::{Offset, OffsetState, LocalResult};

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct UTC1y; // same to UTC but with an offset of 365 days

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct OneYear;

    impl Offset for UTC1y {
        type State = OneYear;

        fn from_state(_state: &OneYear) -> UTC1y { UTC1y }

        fn state_from_local_date(&self, _local: &NaiveDate) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }
        fn state_from_local_time(&self, _local: &NaiveTime) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }
        fn state_from_local_datetime(&self, _local: &NaiveDateTime) -> LocalResult<OneYear> {
            LocalResult::Single(OneYear)
        }

        fn state_from_utc_date(&self, _utc: &NaiveDate) -> OneYear { OneYear }
        fn state_from_utc_time(&self, _utc: &NaiveTime) -> OneYear { OneYear }
        fn state_from_utc_datetime(&self, _utc: &NaiveDateTime) -> OneYear { OneYear }
    }

    impl OffsetState for OneYear {
        fn local_minus_utc(&self) -> Duration { Duration::days(365) }
    }

    impl fmt::Debug for OneYear {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "+8760:00") }
    }

    #[test]
    fn test_date_weird_offset() {
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 2, 29)),
                   "2012-02-29+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 2, 29).and_hms(5, 6, 7)),
                   "2012-02-29T05:06:07+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 3, 4)),
                   "2012-03-04+8760:00".to_string());
        assert_eq!(format!("{:?}", UTC1y.ymd(2012, 3, 4).and_hms(5, 6, 7)),
                   "2012-03-04T05:06:07+8760:00".to_string());
    }
}

