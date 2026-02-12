use crate::config::Settings;
use crate::errors::Result;
use crate::http::HttpClient;

pub struct App {
    pub settings: Settings,
    pub http_client: HttpClient,
}

impl App {
    pub fn new() -> Result<Self> {
        let settings = Settings::new()?;
        let http_client = HttpClient::new();

        Ok(Self {
            settings,
            http_client,
        })
    }

    pub fn initialize(&self) -> Result<()> {
        self.settings.ensure_dirs()?;
        Ok(())
    }
}
