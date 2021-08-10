#[derive(Debug)]
pub struct PrometheusSample<'a> {
    pub name: &'a str,
    pub labels: Option<Vec<(&'a str, &'a str)>>,
    pub value: f64,
    pub timestamp: Option<u64>,
}
