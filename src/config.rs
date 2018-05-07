#[derive(Debug, Deserialize)]
pub struct Config {
    pub show: Option<ShowConfig>,
    pub check: Option<CheckConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CheckConfig {
    pub input: String,
    pub output: Option<String>,
    pub pattern: String,
}

#[derive(Debug, Deserialize)]
pub struct ShowConfig {
    pub input: Option<String>,
    pub format: Option<ShowFormat>,
}

#[derive(Debug, Deserialize)]
pub enum ShowFormat {
    Cli,
}
