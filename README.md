# prometheus-health-checker

A small health check adapter for retrieving and interpreting metrics from prometheus exporter in a process,
and communicating if the process is seen as healthy or not with a process exit code.

Intended to use with kubernetes or docker-compose.

# Features

- Check for "up" metrics, where expected value is 1.0 or above
- Check for "fresh" metrics, where expected value is close to current unix time, but not too far in the past.
- Check multiple metrics at once
- Fast
- Sensible defaults
- When a check fails, it outputs to stderr what exactly failed before exiting with status code 1, eliminating guesswork. Just look at your logs.
- Handles and reports connection failures correctly
- When everything looks as expected, it silently exits with status code 0

# Example

`prom_health_check --check-up cio_service_state --check-fresh last_iteration_time --fresh-for 300`

# Usage

(run with --help) to get the same

```
Prometheus exporter health check

USAGE:
    prom_health_check [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --check-fresh <check-fresh>...    Checks if this metric is "fresh", not too far from current unix time in
                                          seconds.
        --check-up <check-up>...          Checks if this metric is "up", 1.0 or greater
        --fresh-for <fresh-for>           For how many seconds a metric is considered fresh [default: 300]
        --url <url>                       URL of prometheus exporter [default: http://localhost:8091]
```
