mod selectable_list;
mod tui;
mod widget_impl;

use super::log_collector::{LOG_COLLECTOR, LogCollector};
use selectable_list::SelectableList;

use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;

pub struct App<'engine, 'dialogue> {
    runner: diavolo::Runner<'engine, 'dialogue>,
    log_collector: LogCollector,
    prev_log_count: usize,
    log_auto_scroll: bool,
    log_scroll_offset: usize,
    selectable_list: Option<SelectableList>,
    should_quit: bool,
}

impl App<'_, '_> {
    fn update_log_scroll(&mut self) {
        let log_count = self.log_collector.len();
        if log_count > 0 {
            self.log_scroll_offset = log_count - 1;
        }
    }
}

impl<'engine, 'dialogue> App<'engine, 'dialogue>
where
    'engine: 'dialogue,
{
    pub fn new(runner: diavolo::Runner<'engine, 'dialogue>) -> Self {
        Self {
            runner,

            log_collector: LOG_COLLECTOR.clone(),
            prev_log_count: 0,
            log_scroll_offset: 0,
            log_auto_scroll: true,

            selectable_list: None,

            should_quit: false,
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        f.render_widget(self, f.area());
    }

    fn update(&mut self, action: Action) -> Option<Action> {
        use diavolo::Action as RunnerAction;
        match action {
            Action::Quit => {
                self.should_quit = true;
                None
            }
            Action::CheckView => {
                if let Some(view) = self.runner.update_view() {
                    tracing::trace!("View state changed: {:?}", view);
                    self.selectable_list = view.selectable().map(SelectableList::from)
                }
                None
            }
            Action::Select(next) => {
                if let Some(selectable_list) = self.selectable_list.as_mut() {
                    if next {
                        selectable_list.state.select_next();
                    } else {
                        selectable_list.state.select_previous();
                    }
                };
                None
            }
            Action::FastForward
            | Action::SkipMessage
            | Action::Advance
            | Action::ConfirmSelectable => {
                let runner_action = match action {
                    Action::FastForward => RunnerAction::ToggleFastForward,
                    Action::SkipMessage => RunnerAction::Skip,
                    Action::Advance => RunnerAction::Advance,
                    Action::ConfirmSelectable => self
                        .selectable_list
                        .as_ref()
                        .map(|selectable_list| {
                            use selectable_list::ListType;

                            let selected_idx = selectable_list
                                .state
                                .selected()
                                .expect("a selection must exist when confirming selectable");

                            match &selectable_list.r#type {
                                ListType::Confirm(confirm_list) => {
                                    let approved = confirm_list.is_approved(selected_idx);
                                    RunnerAction::Confirm(approved)
                                }
                                ListType::Choice(choice_list) => {
                                    let key = choice_list.selected_key(selected_idx);
                                    RunnerAction::Select(key.clone())
                                }
                            }
                        })
                        .expect("selectable_list must be present when confirming selectable"),
                    _ => unreachable!(),
                };
                self.runner
                    .dispatch(runner_action)
                    .then_some(Action::CheckView)
            }
        }
    }

    fn handle_event(&mut self, event: tui::Event) -> Option<Action> {
        match event {
            tui::Event::Render => Some(Action::CheckView),
            tui::Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('f') => Some(Action::FastForward),
                KeyCode::Tab => Some(Action::SkipMessage),
                KeyCode::Enter => self
                    .selectable_list
                    .is_some()
                    .then_some(Action::ConfirmSelectable)
                    .or(Some(Action::Advance)),
                k @ (KeyCode::Up | KeyCode::Down) => {
                    self.selectable_list.as_ref().map(|_| match k {
                        KeyCode::Up => Action::Select(false),
                        KeyCode::Down => Action::Select(true),
                        _ => unreachable!(),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub async fn run(&'engine mut self) -> Result<()> {
        let mut tui = tui::Tui::new()?.tick_rate(4.0).frame_rate(60.0);

        tui.enter()?;

        loop {
            tui.draw(|f| {
                // Deref allows calling `tui.terminal.draw`
                self.ui(f);
            })?;

            if let Some(evt) = tui.next().await {
                let mut maybe_action = self.handle_event(evt);
                while let Some(action) = maybe_action {
                    maybe_action = self.update(action);
                }
            };

            if self.should_quit {
                break;
            }
        }

        tui.exit()?; // stops event handler, exits raw mode, exits alternate screen

        Ok(())
    }
}

enum Action {
    Quit,
    CheckView,
    FastForward,
    SkipMessage,
    Advance,
    Select(bool),
    // Confirm,
    ConfirmSelectable,
}
