//! Generators for tests.
//!
//! Each generator corresponds to a different test.

use super::utils::*;
use crate::types::{Attribute, DateTime, Tag, Value};

use base64 as b64;

use chrono::{Datelike, NaiveDate, NaiveTime};
use chrono::{Local, Offset, TimeZone, Utc};

use itertools::EitherOrBoth;
use itertools::Itertools;

use proptest::prelude::*;
use proptest::strategy::ValueTree;

use std::time::Duration;

/// Creates a string test.
pub fn string(
) -> impl Strategy<Value = Test<String>, Tree = impl ValueTree<Value = Test<String>>>
{
    ("[^\\\\]*", ".*", prop::bool::ANY).prop_map(|(text, escapes, raw)| {
        let ch = if raw { '`' } else { '"' };
        let input = text
            .chars()
            .filter(|c| *c != ch)
            .chunks(2)
            .into_iter()
            .zip_longest(escapes.chars().filter(|c| if raw {
                *c != '`'
            } else {
                true
            }))
            .format_with("", |e, f| match e {
                EitherOrBoth::Left(c) => f(&c.format("")),
                EitherOrBoth::Right(e) => f(&format_args!("\\{}", e)),
                EitherOrBoth::Both(c, e) => {
                    f(&format_args!("{}\\{}", c.format(""), e))
                }
            })
            .to_string();
        Test::new(
            format!("{0:}{1:}{0:}", ch, input),
            if raw {
                input
            } else {
                text.chars()
                    .filter(|c| *c != ch)
                    .chunks(2)
                    .into_iter()
                    .zip_longest(escapes.chars().map(|ch| match ch {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '0' => '\x00',
                        chr => chr,
                    }))
                    .format_with("", |e, f| match e {
                        EitherOrBoth::Left(c) => f(&c.format("")),
                        EitherOrBoth::Right(e) => f(&format_args!("{}", e)),
                        EitherOrBoth::Both(c, e) => {
                            f(&format_args!("{}{}", c.format(""), e))
                        }
                    })
                    .to_string()
            },
        )
    })
}

/// Creates a date test.
pub fn date() -> impl Strategy<
    Value = Test<NaiveDate>,
    Tree = impl ValueTree<Value = Test<NaiveDate>>,
> {
    (0i32..10_000, 1u32..=12)
        .prop_flat_map(|(y, m)| {
            let day = NaiveDate::from_ymd(y, m, 1).pred();
            (Just(day.year()), Just(day.month()), 1..=day.day())
        })
        .prop_map(|(y, m, d)| {
            Test::new(
                format!("{:04}/{:02}/{:02}", y, m, d),
                NaiveDate::from_ymd(y, m, d),
            )
        })
}

/// Creates a time test.
pub fn time() -> impl Strategy<
    Value = Test<NaiveTime>,
    Tree = impl ValueTree<Value = Test<NaiveTime>>,
> {
    prop::bool::ANY
        .prop_flat_map(|with_ms| {
            (
                Just(with_ms),
                (0u32..24, 0u32..60, 0u32..60, 0u32..1000).prop_map(
                    move |(h, m, s, ms)| {
                        if with_ms {
                            NaiveTime::from_hms_milli(h, m, s, ms)
                        } else {
                            NaiveTime::from_hms(h, m, s)
                        }
                    },
                ),
            )
        })
        .prop_map(|(with_ms, time)| {
            Test::new(
                time.format(if with_ms { "%H:%M:%S%.3f" } else { "%H:%M:%S" })
                    .to_string(),
                time,
            )
        })
}

/// Creates a datetime test.
pub fn datetime() -> impl Strategy<
    Value = Test<DateTime>,
    Tree = impl ValueTree<Value = Test<DateTime>>,
> {
    (date(), time(), "[ \t]+", prop::bool::ANY).prop_map(
        |(date, time, white, utc)| {
            Test::new(
                format!(
                    "{}{}{}{}",
                    date.text,
                    white,
                    time.text,
                    if utc { "-UTC" } else { "" }
                ),
                if utc {
                    Utc.fix()
                        .from_utc_datetime(&date.result.and_time(time.result))
                } else {
                    let naive = date.result.and_time(time.result);
                    let local = Local.from_local_datetime(&naive).unwrap();
                    local.with_timezone(local.offset())
                },
            )
        },
    )
}

/// Creates a duration test.
pub fn duration() -> impl Strategy<
    Value = Test<Duration>,
    Tree = impl ValueTree<Value = Test<Duration>>,
> {
    (
        prop::bool::ANY,
        prop::bool::ANY,
        0u64..24,
        0u64..60,
        0u64..60,
        prop::num::u32::ANY,
        0u32..1000,
    )
        .prop_map(|(with_d, with_ms, h, m, s, d, ms)| {
            Test::new(
                format!(
                    "{}{:02}:{:02}:{:02}{}",
                    if with_d {
                        format!("{}d:", d)
                    } else {
                        String::new()
                    },
                    h,
                    m,
                    s,
                    if with_ms {
                        format!(".{:03}", ms)
                    } else {
                        String::new()
                    }
                ),
                Duration::new(
                    s + 60
                        * (m + 60
                            * (h + 24 * u64::from(if with_d { d } else { 0 }))),
                    1_000_000 * if with_ms { ms } else { 0 },
                ),
            )
        })
}

/// Creates a number test.
pub fn number(
) -> impl Strategy<Value = Test<i128>, Tree = impl ValueTree<Value = Test<i128>>>
{
    (prop::num::i128::ANY, 0u8..3).prop_map(|(n, s)| {
        let (n, suf) = match s {
            0 => (i128::from(n as i32), ""),
            1 => (i128::from(n as i64), "L"),
            2 => (n, "BD"),
            _ => unreachable!(),
        };
        Test::new(format!("{}{}", n, suf), n)
    })
}

/// Creates a decimal test.
pub fn decimal(
) -> impl Strategy<Value = Test<f64>, Tree = impl ValueTree<Value = Test<f64>>>
{
    (-1e20f64..1e20, 1..std::f64::DIGITS).prop_map(|(n, s)| {
        let text = format!("{:.*}", s as usize, n);
        let (n, suf) = if s < std::f32::DIGITS {
            (text.parse::<f32>().unwrap() as f64, "f")
        } else {
            (text.parse::<f64>().unwrap(), "")
        };
        Test::new(format!("{:.*}{}", s as usize, n, suf), n)
    })
}

/// Creates a boolean test.
pub fn boolean(
) -> impl Strategy<Value = Test<bool>, Tree = impl ValueTree<Value = Test<bool>>>
{
    (prop::bool::ANY, prop::bool::ANY).prop_map(|(v, s)| {
        Test::new(
            match (v, s) {
                (true, true) => "true",
                (true, false) => "on",
                (false, true) => "false",
                (false, false) => "off",
            }
            .to_string(),
            v,
        )
    })
}

/// Creates a base64 test.
pub fn base64(
) -> impl Strategy<Value = Test<Vec<u8>>, Tree = impl ValueTree<Value = Test<Vec<u8>>>>
{
    (0usize..256)
        .prop_flat_map(|size| {
            (
                prop::collection::vec(prop::num::u8::ANY, size),
                prop::collection::vec("[ \t]*", (size + 2) / 3),
            )
        })
        .prop_map(|(bytes, whites)| {
            Test::new(
                format!(
                    "[{}]",
                    b64::encode(&bytes)
                        .drain(..)
                        .chunks(4)
                        .into_iter()
                        .zip(whites.into_iter())
                        .format_with("", |(chars, white), f| f(&format_args!(
                            "{}{}",
                            chars.format(""),
                            white
                        )))
                ),
                bytes,
            )
        })
}

/// Creates a null test.
pub fn null(
) -> impl Strategy<Value = Test<()>, Tree = impl ValueTree<Value = Test<()>>> {
    Just(Test::new("null".into(), ()))
}

/// Creates a value test.
pub fn value(
) -> impl Strategy<Value = Test<Value>, Tree = impl ValueTree<Value = Test<Value>>>
{
    prop_oneof![
        null().prop_map(|test| test.map_res(|()| Value::Null)),
        boolean().prop_map(|test| test.map_res(Value::from)),
        number().prop_map(|test| test.map_res(Value::from)),
        decimal().prop_map(|test| test.map_res(Value::from)),
        date().prop_map(|test| test.map_res(Value::from)),
        datetime().prop_map(|test| test.map_res(Value::from)),
        duration().prop_map(|test| test.map_res(Value::from)),
        string().prop_map(|test| test.map_res(Value::from)),
        base64().prop_map(|test| test.map_res(Value::from)),
    ]
}

/// Creates an ident strategy.
pub fn ident(
) -> impl Strategy<Value = Test<String>, Tree = impl ValueTree<Value = Test<String>>>
{
    "[a-zA-Z_][a-zA-Z0-9.$_-]*".prop_filter_map("Boolean/Null", |txt| match txt
        .as_str()
    {
        "on" | "off" | "true" | "false" | "null" => None,
        _ => Some(Test::new(txt.clone(), txt)),
    })
}

/// Creates an attribute strategy.
pub fn attribute() -> impl Strategy<
    Value = Test<Attribute>,
    Tree = impl ValueTree<Value = Test<Attribute>>,
> {
    (ident(), value()).prop_map(|(ident, value)| {
        Test::new(
            format!("{}={}", ident.text, value.text),
            Attribute::new(ident.result, value.result),
        )
    })
}

/// Creates a namespace strategy.
pub fn namespace(
) -> impl Strategy<Value = Test<String>, Tree = impl ValueTree<Value = Test<String>>>
{
    ident().prop_map(|test| test.map_text(|text| text + ":"))
}

/// Creates a (recursive!) tag strategy.
pub fn tag(
) -> impl Strategy<Value = Test<Tag>, Tree = impl ValueTree<Value = Test<Tag>>>
{
    tag_minimal().prop_recursive(8, 256, 8, |elem| {
        (
            tag_minimal(),
            prop::collection::vec((elem, "[ \t]*[;\n][ \t]*"), 0..8),
            "[ \t]+",
        )
            .prop_map(|(tag, elems, white)| {
                Test::new(
                    format!(
                        "{}{{{}{}}}",
                        tag.text,
                        white,
                        elems.iter().format_with("", |(t, w), f| f(
                            &format_args!("{}{}", t.text, w)
                        ))
                    ),
                    tag.result.tags(elems.into_iter().map(|e| e.0.result)),
                )
            })
    })
}

/// Creates a minimal, non-recursive tag.
fn tag_minimal(
) -> impl Strategy<Value = Test<Tag>, Tree = impl ValueTree<Value = Test<Tag>>>
{
    (
        prop::bool::ANY, prop::option::of(namespace()), ident(), "[ \t]+",
        prop::collection::vec((value(), "[ \t]+"), 0..32),
        prop::collection::vec((attribute(), "[ \t]+"), 0..32),
    )
    .prop_map(|(use_name, namespace, name, white, values, attributes)| {
        let name = if values.len() == 0 || use_name {
            let namespace = namespace
                .map_or(Test::new(String::new(), None), |namespace| {
                    namespace.map_res(Some)
                });
            Test::new(
                format!("{}{}{}", namespace.text, name.text, white),
                Tag::new(name.result).namespace_opt(namespace.result),
            )
        } else {
            Test::new(String::new(), Tag::new(String::new()))
        };
        Test::new(
            format!(
                "{}{}{}",
                name.text,
                values.iter().format_with("", |(v, w), f| f(
                    &format_args!("{}{}", v.text, w,)
                )),
                attributes.iter().format_with("", |(a, w), f| f(
                    &format_args!("{}{}", a.text, w,)
                ))
            ),
            name.result
                .values(values.into_iter().map(|e| e.0.result))
                .attrs(attributes.into_iter().map(|e| e.0.result)),
        )
    })
}

/// Creates a tagtree test.
pub fn tagtree() -> impl Strategy<
    Value = Test<Vec<Tag>>,
    Tree = impl ValueTree<Value = Test<Vec<Tag>>>,
> {
    prop::collection::vec((tag(), "[ \t]*[;\n][ \t]*"), 0..8).prop_map(|tags| {
        Test::new(
            tags.iter()
                .format_with("", |(t, w), f| {
                    f(&format_args!("{}{}", t.text, w))
                })
                .to_string(),
            tags.into_iter().map(|e| e.0.result).collect(),
        )
    })
}
