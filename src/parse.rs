//! Implements low-level parsing routines.
//!
//! These routines parse the given parse tree into a specific Rule, as
//! determined by the function name.

use base64 as b64;

use chrono::{Timelike, NaiveTime, NaiveDateTime};
use chrono::{Offset, Utc, Local, TimeZone};

use crate::{Result, Attribute, Value, Tag, Date, DateTime};
use crate::grammar::{Rule, ParseTree, parse_err};

use std::time::Duration;

pub fn string(tree: ParseTree) -> Result<String> {
    // Get positional info
    let span = tree.as_span();
    let beg = span.start() + 1;
    let end = span.end() - 1;
    let len = end - beg;

    let text = &tree.as_str()[1..=len];

    // Check if its a raw string. If so, then we don't parse escapes, so exit.
    if tree.as_str().chars().next().unwrap() == '`' {
        return Ok(text.to_string());
    }

    // Iterate, parsing escapes.
    Ok(text.chars().fold((false, String::with_capacity(len)),
        |(esc, mut res), ch| if esc {
            // Parsing an escape.
            res.push(match ch {
                '\n' | 'n' => '\n',
                'r'  => '\r',
                't'  => '\t',
                '\\' => '\\',
                '0'  => '\x00',
                '"'  => '"',
                '\'' => '\'',
                _ => unreachable!()
            });
            (false, res)
        } else if ch == '\\' {
            // It's an escape. Don't add it to the string.
            (true, res)
        } else {
            res.push(ch);
            (false, res)
        }
    ).1)
}

pub fn date(tree: ParseTree) -> Result<Date> {
    Date::parse_from_str(tree.as_str(), "%Y/%m/%d")
        .map_err(|_| parse_err(
            format!("Error in parsing '{}' into a date!", tree.as_str()),
            tree.as_span()
        ))
}

pub fn time(tree: ParseTree) -> Result<NaiveTime> {
    let text = tree.as_str();
    let span = tree.as_span();
    let fmt = if tree.into_inner().next().is_some() {
        "%H:%M:%S%.3f"
    } else { "%H:%M:%S" };

    NaiveTime::parse_from_str(text, fmt)
        .map_err(|_| parse_err(
            format!("Error in parsing '{}' into a time!", text),
            span
        ))
}

pub fn datetime(tree: ParseTree) -> Result<DateTime> {
    let mut pairs = tree.into_inner();
    let date = date(pairs.next().unwrap())?;
    let time = time(pairs.next().unwrap())?;

    let naive = NaiveDateTime::new(date, time);

    Ok(if pairs.next().is_some() {
        Utc.fix().from_utc_datetime(&naive)
    } else {
        let local = Local.from_local_datetime(&naive).unwrap();
        local.with_timezone(local.offset())
    })
}

pub fn duration(tree: ParseTree) -> Result<Duration> {
    let mut dur: Duration = Duration::new(0,0);

    tree.into_inner().try_for_each(|p| match p.as_rule() {
        Rule::days => p.as_str().parse::<u32>().map(|val| {
                dur += Duration::from_secs(val as u64 * 24 * 60 * 60);
            }).map_err(|_| parse_err(
                format!("Could not parse days from '{}'!", p.as_str()),
                p.as_span()
            )),
        Rule::time => time(p).map(|time| {
            dur += Duration::new(time.second() as u64 + 60 * (
                    time.minute() as u64  + 60 * time.hour() as u64),
                    time.nanosecond());
        }),
        _ => unreachable!()
    })?;

    Ok(dur)
}

pub fn number(tree: ParseTree) -> Result<i128> {
    let mut pairs = tree.into_inner();

    let num = pairs.next().unwrap();
    let text = num.as_str();

    match pairs.next().map(|p| p.as_str()) {
        None => text.parse::<i32>().map(|n| n as i128),
        Some("L") => text.parse::<i64>().map(|n| n as i128),
        Some("BD") => text.parse::<i128>(),
        _ => unreachable!(),
    }.map_err(|_| parse_err(
        format!("Error in parsing '{}' as a number (too large?)", text),
        num.as_span()
    ))
}

pub fn decimal(tree: ParseTree) -> Result<f64> {
    let mut pairs = tree.into_inner();

    let num = pairs.next().unwrap();
    let text = num.as_str();

    match pairs.next().map(|p| p.as_str()) {
        None => text.parse::<f32>().map(|n| n as f64),
        Some("f") => text.parse::<f64>(),
        _ => unreachable!(),
    }.map_err(|_| parse_err(
        format!("Error in parsing '{}' as a decimal (too large?)", text),
        num.as_span()
    ))
}

pub fn boolean(tree: ParseTree) -> Result<bool> {
    Ok(match tree.into_inner().next().unwrap().as_rule() {
        Rule::bool_true     => true,
        Rule::bool_false    => false,
        _                   => unreachable!()
    })
}

pub fn base64(tree: ParseTree) -> Result<Vec<u8>> {
    let span = tree.as_span();
    b64::decode(tree.into_inner()
            .flat_map(|ch| ch.as_str().bytes())
            .collect::<Vec<_>>().as_slice())
        .map_err(|e| parse_err(
            format!("Error in parsing Base64: {}", e),
            span
        ))
}

pub fn value(tree: ParseTree) -> Result<Value> {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::string    => string(tree).map(|v| v.into()),
        Rule::base64    => base64(tree).map(|v| v.into()),
        Rule::date      => date(tree).map(|v| v.into()),
        Rule::datetime  => datetime(tree).map(|v| v.into()),
        Rule::duration  => duration(tree).map(|v| v.into()),
        Rule::number    => number(tree).map(|v| v.into()),
        Rule::decimal   => decimal(tree).map(|v| v.into()),
        Rule::boolean   => boolean(tree).map(|v| v.into()),
        Rule::null      => Ok(Value::Null),
        _               => unreachable!(),
    }
}

pub fn ident(tree: ParseTree) -> Result<String> {
    Ok(tree.as_str().to_string())
}

pub fn attribute(tree: ParseTree) -> Result<Attribute> {
    let mut pairs = tree.into_inner();
    let name = pairs.next().unwrap();
    let val = pairs.next().unwrap();
    Ok((ident(name)?, value(val)?).into())
}

pub fn namespace(tree: ParseTree) -> Result<String> {
    ident(tree.into_inner().next().unwrap())
}

pub fn tag(tree: ParseTree) -> Result<Tag> {
    tree.into_inner().try_fold(Tag::new(String::new()), |mut tag, tree| {
        match tree.as_rule() {
            Rule::namespace => tag.namespace = Some(namespace(tree)?),
            Rule::ident     => tag.name = ident(tree)?,
            Rule::value     => tag.values.push(value(tree)?),
            Rule::attribute => tag.attrs.push(attribute(tree)?),
            Rule::tags      => tag.tags.append(&mut tags(tree)?),
            _               => unreachable!()
        }
        Ok(tag)
    })
}

pub fn tags(tree: ParseTree) -> Result<Vec<Tag>> {
    tree.into_inner().map(|tree| tag(tree)).collect()
}

pub fn tagtree(tree: ParseTree) -> Result<Vec<Tag>> {
    tags(tree.into_inner().next().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::grammar::{Rule, parse};
    use crate::parse::*;

    #[test]
    pub fn test_tag() {
        let test = r#"hello_world "string" 1 true attr=on {hi;bye}"#;
        assert_eq!(
            parse(Rule::tag, test).and_then(tag),
            Ok(Tag::new("hello_world".into())
                .values(vec!["string".into(), 1.into(), true.into()])
                .attrs(vec![("attr".to_string(), true.into()).into()])
                .tags(vec![
                    Tag::new("hi".into()),
                    Tag::new("bye".into()),
                ])
            )
        );
    }
}
