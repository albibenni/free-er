# TODO

## UI

- [x] sidebar:
  - [x] icons
  - [x] toggle sidebar with a button - if closed, only show icons, if open show icons and text
  - [x] settings on the bottom, not in the middle
- [x] accent color:
  - [x] allow user to choose from a palette of colors
  - [x] use accent color for buttons, icons, highlights
- [ ] dark light mode:
  - [ ] automatic based on system settings
  - [ ] manual toggle in settings

## LOGIC

- [x] wildcard management:
  - [x] * means all sites: eg *.com means all .com sites, but not .com.br, youtube.com/watch* means all youtube.com/watch pages, but not youtube.com/channel
- [x] allow all search engines from settings
- [x] allow all ai web pages from settings
- [x] allow new tab page from settings
- [x] allow localhosts and ips

## FEATURES

- [x] open at startup

### STRICT MODE

- [ ] block all toggle in settings
  - [ ] disable strict mode require a confirmation dialog with a warning and a call to action to disable it
- [ ] calendar settings are untoggable if strict mode is on
- [ ] calendar settings lists are disabled if strict mode is on
- [ ] pomodoro require a confirmation dialog with a warning and a call to stop it
- [ ] quick break require a confirmation dialog with a warning and a call to action to disable strict mode
- [ ] allowed list add with strict mode on should require a confirmation dialog with a warning and a call to action to add a new allowed site
  - [ ] same for adding a website from 'Open tabs'
- [ ] schedule task require a confirmation dialog with a warning and a call to action to create a new task if strict mode is on to change the list of allowed sites for that task or from focus mode to break mode (and vice versa)
- [ ] schedule task cannot be modified if strict mode is on - changing the time or the allowed list should require a confirmation dialog with a warning and a call to action to modify the task
- [ ] schedule task cannot be deleted if strict mode is on - deleting a task should require a confirmation dialog with a warning and a call to action to delete the task
- [ ] schedule cannnot be created if strict mode is on - creating a task should require a confirmation dialog with a warning and a call to action to create the task
  
### CALENDAR

- [x] allow modify existing schedules from calendar ui
- [x] snap 15 min also when creating new task (not only when modifying)
- [x] task should show start and end time
- [x] adding new task should overlay start time and end time while selecting
- [x] two task in the same time - split the space
- [x] allow drag and drop to change the time of the task
- [x] task should have selection for focus allowed list
- [x] task could be a break
- [x] by default should take the default allowed list, not none
- [x] icon and text style:
  - [x] icon padding from title
  - [x] icon and text bigger
- [x] resync with google calendar when the app starts
- [x] resync button to resync with google calendar
- [x] import from calendar previous week and next week only
  - [x] clean schedule out of time - 2 weeks ago and 2 weeks in the future
  - [x] check if weekly could affect it
- [x] allow creating inside repeating task
- [x] modify repeating days
- [x] see repreating days in calendar imports too - but blocked from modifying them
- [x] change default allowed list from settings
- [x] import from open websites
- [x] rules for import
  - [x] personalized list
    - [x] no screen
    - [x] focus, study, work
    - [x] break, free, colloquio
- [x] calendar view scrollable - focussed to current time

### POMODORO

- [x] pomodoro timer clock
- [x] snap 5 min drag
- [x] opposite - clock wise to increase, counter clock wise to decrease
- [ ] break timer clock - connected to the pomodoro end
- [x] race condition between pomodoro and calendar task - pomodoro is above calendar task, but when pomodoro ends, calendar task should come back
- [ ] pomodoro list selection cannot be changed while pomodoro is active
- [ ] pomodoro cannot be increased or decreased while active (both for focus and break)
- [ ] disable stop if pomodoro isn't active

## BUG

- [x] allowed list not working
- [x] focus button not responding - responsivness issue
- [x] allowed list doesn't show the list of allowed sites
- [ ] why clone on sender in app.rs
- [x] save schedule doesn't save the schedule on the calendar
- [ ] break isn't actual break - it is still in focus
- [x] thread 'tokio-rt-worker' (285106) panicked at /home/albibenni/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gtk4-0.10.3/src/auto/css_provider.rs:30:9:
GTK may only be used from the main thread.

keeps paniking

- [x] ui thread connected to the deamon thread - if I close the ui, the daemon should stop, if I close the daemon, the ui should stop

## OPTIMIZATION

- [x] pomodoro.rs refactor - too much code, should be split into smaller functions
- [x] event loop - use a more efficient way to handle events, maybe with a channel or a queue, instead of matching on every event
  - [x] risk: Lock discipline in the daemon: emit() must always be called after releasing the `Mutex<Inner>` guard, never while holding it. The fix is a small emit() helper that clones the Sender out of the lock first, then sends outside.
