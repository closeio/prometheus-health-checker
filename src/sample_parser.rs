use crate::sample::PrometheusSample;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::{digit1, space1};
use nom::character::is_alphanumeric;
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, separated_pair, tuple};

// https://prometheus.io/docs/instrumenting/exposition_formats/#text-format-details
// metric_name [
//   "{" label_name "=" `"` label_value `"` { "," label_name "=" `"` label_value `"` } [ "," ] "}"
// ] value [ timestamp ]

fn integer(input: &str) -> nom::IResult<&str, u64> {
    map_res(digit1, |s: &str| s.parse::<u64>())(input)
}

fn label_value(input: &str) -> nom::IResult<&str, &str> {
    delimited(tag("\""), take_until("\""), tag("\""))(input)
}

fn is_id_char(chr: char) -> bool {
    is_alphanumeric(chr as u8) || chr == '_'
}

fn identifier(input: &str) -> nom::IResult<&str, &str> {
    take_while1(is_id_char)(input)
}

fn label(input: &str) -> nom::IResult<&str, Vec<(&str, &str)>> {
    separated_list0(tag(","), separated_pair(identifier, tag("="), label_value))(input)
}

pub fn parse_prometheus_sample(input: &str) -> nom::IResult<&str, PrometheusSample> {
    map(
        all_consuming(tuple((
            identifier,
            opt(delimited(tag("{"), label, tag("}"))),
            preceded(space1, double),
            opt(preceded(space1, integer)),
        ))),
        |(name, labels, value, timestamp)| PrometheusSample {
            name,
            labels,
            value,
            timestamp,
        },
    )(input)
}
