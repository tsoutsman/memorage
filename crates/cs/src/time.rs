//! This module has been ripped straight out of [`times`][time] internals - with
//! some minor modifications. [`time`][time] internally uses [`deserialize_any`]
//! which doesn't work with [`bincode`]. This could probably be accomplished by
//! an upstream change but I'm too scared to touch other people's serde code.
//!
//! [time]: memorage_core::time
//! [`deserialize_any`]: serde::Deserializer::deserialize_any

use memorage_core::time::{Date, OffsetDateTime, UtcOffset};
use serde::{de, Serialize};

pub(crate) fn serialize_offset_date_time<S>(
    time: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    (
        time.year(),
        time.ordinal(),
        time.hour(),
        time.minute(),
        time.second(),
        time.nanosecond(),
        time.offset().whole_hours(),
        time.offset().minutes_past_hour(),
        time.offset().seconds_past_minute(),
    )
        .serialize(serializer)
}

macro_rules! item {
    ($seq:expr, $name:literal) => {
        $seq.next_element()?
            .ok_or_else(|| <A::Error as serde::de::Error>::custom(concat!("expected ", $name)))
    };
}

pub(crate) fn deserialize_offset_date_time<'de, D>(
    deserializer: D,
) -> Result<OffsetDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OffsetDateTimeVisitor;

    impl<'a> serde::de::Visitor<'a> for OffsetDateTimeVisitor {
        type Value = OffsetDateTime;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an offset date time")
        }

        fn visit_seq<A: de::SeqAccess<'a>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let year = item!(seq, "year")?;
            let ordinal = item!(seq, "day of year")?;
            let hour = item!(seq, "hour")?;
            let minute = item!(seq, "minute")?;
            let second = item!(seq, "second")?;
            let nanosecond = item!(seq, "nanosecond")?;
            let offset_hours = item!(seq, "offset hours")?;
            let offset_minutes = item!(seq, "offset minutes")?;
            let offset_seconds = item!(seq, "offset seconds")?;

            Date::from_ordinal_date(year, ordinal)
                .and_then(|date| date.with_hms_nano(hour, minute, second, nanosecond))
                .and_then(|datetime| {
                    UtcOffset::from_hms(offset_hours, offset_minutes, offset_seconds)
                        .map(|offset| datetime.assume_offset(offset))
                })
                .map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_tuple(9, OffsetDateTimeVisitor)
}
