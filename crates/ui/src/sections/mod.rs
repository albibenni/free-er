pub mod allowed_lists;
pub mod calendar_rules;
pub mod focus;
pub mod pomodoro;
pub mod schedule;
pub mod settings;

#[cfg(test)]
pub(crate) mod test_support {
    pub static GTK_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
}
