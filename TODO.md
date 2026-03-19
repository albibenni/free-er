# TODO

## UI

- [ ] sidebar:
  - [x] icons
  - [x] toggle sidebar with a button - if closed, only show icons, if open show icons and text
  - [x] settings on the bottom, not in the middle

## LOGIC

- [x] wildcard management:
  - [x] * means all sites: eg *.com means all .com sites, but not .com.br, youtube.com/watch* means all youtube.com/watch pages, but not youtube.com/channel
- [x] allow all search engines from settings
- [x] allow all ai web pages from settings
- [x] allow new tab page from settings

## FEATURES

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
  - [ ] check if weekly could affect it
- [ ] allow creating inside repeating task
- [ ] change default allowed list from settings
- [ ] import from open websites
- [x] rules for import
  - [x] personalized list
    - [x] no screen
    - [x] focus, study, work
    - [x] break, free, colloquio

### POMODORO

- [ ] pomodoro timer clock
- [ ] break timer clock - connected to the pomodoro end
- [ ] race condition between pomodoro and calendar task - pomodoro is above calendar task, but when pomodoro ends, calendar task should come back

## BUG

- [x] allowed list not working
- [x] focus button not responding - responsivness issue
- [x] allowed list doesn't show the list of allowed sites
- [ ] why clone on sender in app.rs
- [x] save schedule doesn't save the schedule on the calendar
- [ ] break isn't actual break - it is still in focus
