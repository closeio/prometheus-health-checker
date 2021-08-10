use crate::checks::{Check, CheckContext, CheckType};
use crate::sample_parser::parse_prometheus_sample;
use clap::{App, Arg};
use std::collections::BTreeSet;
use std::io;
use std::io::{Error, ErrorKind};
use std::process::exit;

mod checks;
mod sample;
mod sample_parser;

fn main() {
    if let Err(e) = health_check() {
        eprintln!("{}", e);
        exit(1);
    }
}

fn health_check() -> Result<(), io::Error> {
    let args = App::new("Prometheus exporter health check")
        .arg(
            Arg::with_name("url")
                .long("url")
                .help("URL of prometheus exporter")
                .default_value("http://localhost:8091")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fresh-for")
                .long("fresh-for")
                .help("For how many seconds a metric is considered fresh")
                .default_value("300")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("check-fresh")
                .long("check-fresh")
                .multiple(true)
                .takes_value(true)
                .help("Checks if this metric is \"fresh\", not too far from current unix time in seconds."),
        )
        .arg(
            Arg::with_name("check-up")
                .long("check-up")
                .multiple(true)
                .takes_value(true)
                .help("Checks if this metric is \"up\", 1.0 or greater"),
        )
        .get_matches();

    let url = args.value_of("url").ok_or(string_error("No url value"))?;
    let fresh_for: f64 = args
        .value_of("fresh-for")
        .ok_or(string_error("No fresh-for value"))?
        .parse()
        .map_err(|e| string_error(e))?;
    let context = CheckContext::new(fresh_for);
    let mut checks: Vec<Check> = vec![];

    for (arg, check_type) in vec![
        ("check-up", CheckType::Up),
        ("check-fresh", CheckType::Fresh),
    ] {
        checks.extend(
            args.values_of(arg)
                .into_iter()
                .flatten()
                .map(|name| Check { name, check_type }),
        );
    }

    let body: String = retrieve_url(url)?;

    match_metrics(context, &mut checks, &body)
}

fn match_metrics(context: CheckContext, checks: &[Check], body: &str) -> Result<(), Error> {
    let mut confirmed_metrics: BTreeSet<&str> = BTreeSet::new();
    for sample in body
        .lines()
        .filter(|l| !l.starts_with('#'))
        .flat_map(parse_prometheus_sample)
        .map(|r| r.1)
    {
        let matching_checks: Vec<&Check> =
            checks.iter().filter(|c| c.name == sample.name).collect();

        confirmed_metrics.extend(matching_checks.iter().map(|c| c.name));

        let failed_checks: Vec<&Check> = matching_checks
            .into_iter()
            .filter(|c| !c.is_satisfied_by(&sample, context))
            .collect();

        if !failed_checks.is_empty() {
            return Err(string_error(format!(
                "Metric {} with value {} failed checks: {:?}",
                sample.name, sample.value, failed_checks
            )));
        }
    }
    let requested_metrics: BTreeSet<&str> = checks.iter().map(|c| c.name).collect();
    if !confirmed_metrics.is_superset(&requested_metrics) {
        return Err(string_error(format!(
            "The following metrics were not found: {:?}",
            requested_metrics
                .difference(&confirmed_metrics)
                .map(|s| *s)
                .collect::<Vec<&str>>()
        )));
    }
    Ok(())
}

fn string_error(s: impl ToString) -> io::Error {
    io::Error::new(ErrorKind::Other, s.to_string())
}

fn retrieve_url(url: &str) -> Result<String, Error> {
    ureq::get(url)
        .call()
        .map_err(|e| string_error(e))
        .and_then(|r| r.into_string())
}

#[cfg(test)]
mod tests {
    use crate::checks::{Check, CheckContext, CheckType};
    use crate::match_metrics;

    #[test]
    fn it_matches_metrics() {
        let context = CheckContext {
            now: 100.0,
            stale_threshold: 10.0,
        };
        let checks = vec![
            Check {
                name: "up",
                check_type: CheckType::Up,
            },
            Check {
                name: "now",
                check_type: CheckType::Fresh,
            },
        ];
        assert!(match_metrics(context, &checks, "up 1\nnow 100").is_ok());
        assert!(match_metrics(context, &checks, "up 1\nnow 100\n\n").is_ok());
        assert!(match_metrics(context, &checks, "extra 0\nup 1\nnow 100").is_ok());
        assert!(match_metrics(context, &checks, "up 2\nnow 100").is_ok());
        assert!(match_metrics(context, &checks, "up 1\nnow 95").is_ok());

        assert!(match_metrics(context, &checks, "up 0\nnow 100").is_err());
        assert!(match_metrics(context, &checks, "up 1\nnow 80").is_err());
        assert!(match_metrics(context, &checks, "missingup 1\nnow 100").is_err());
        assert!(match_metrics(context, &checks, "up 1\nmissingnow 100").is_err());
        assert!(match_metrics(context, &checks, "").is_err());
    }
}
