#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsInput {
    SetStrictMode(bool),
    SetAllowNewTab(bool),
    AllowNewTabUpdated(bool),
    SetAiSites(bool),
    SetSearchEngines(bool),
    SetQuick(&'static str, bool),
    QuickUrlsUpdated(Vec<String>),
    SaveCalDav,
    ConnectGoogle,
    DisconnectGoogle,
    GoogleStatusUpdated(bool),
    SetAccentColor(String),
    AccentColorUpdated(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsOutput {
    StrictModeChanged(bool),
    AllowNewTabChanged(bool),
    AiSitesToggled(bool),
    SearchEnginesToggled(bool),
    QuickUrlToggled {
        url: &'static str,
        enabled: bool,
    },
    CalDavSaved {
        url: String,
        user: String,
        pass: String,
    },
    ConnectGoogleRequested,
    DisconnectGoogleRequested,
    AccentColorChanged(String),
}
