mod choice_list;
mod log_collector;
mod tui;
mod widget_impl;

use choice_list::ChoiceList;
use diavolo::{Dialogue, Runner, View};
use log_collector::{CollectorLayer, LogCollector};

use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::Frame;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct App<'engine, 'dialogue> {
    runner: diavolo::Runner<'engine, 'dialogue>,
    view_cache: View<'static>,
    log_collector: LogCollector,
    prev_log_count: usize,
    log_auto_scroll: bool,
    log_scroll_offset: usize,
    choice_list: Option<ChoiceList>,
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

impl<'engine, 'dialogue> App<'engine, 'dialogue> {
    pub fn new(runner: Runner<'engine, 'dialogue>) -> Self {
        let log_collector = LogCollector::new(1000);
        tracing_subscriber::registry()
            .with(CollectorLayer::new(log_collector.clone()))
            .init();

        Self {
            runner,
            view_cache: View::default(),

            log_collector,
            prev_log_count: 0,
            log_scroll_offset: 0,
            log_auto_scroll: true,

            choice_list: None,

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
            }
            Action::CheckView => {
                let view = self.runner.view();
                if &self.view_cache != &view {
                    tracing::trace!("View state changed: {:?}", view);
                    self.view_cache = view.into_owned();
                    if self.view_cache.has_message_finished() {
                        return Some(Action::CompleteMessage);
                    }
                    self.choice_list = self
                        .view_cache
                        .choices()
                        .map(|choices| ChoiceList::from_iter(choices));
                }
            }
            Action::FastForward => {
                self.runner.dispatch(RunnerAction::ToggleFastForward);
            }
            Action::SkipMessage => {
                self.runner.dispatch(RunnerAction::Skip);
            }
            Action::CompleteMessage => {
                self.runner
                    .dispatch(RunnerAction::CompleteMessage);
            }
            Action::NextLine => {
                self.runner
                    .dispatch(RunnerAction::NextLine);
            }
            Action::SelectChoice(next) => {
                self.choice_list.as_mut().map(|choice_list| {
                    if next {
                        choice_list.state.select_next();
                    } else {
                        choice_list.state.select_previous();
                    }
                });
            }
            Action::ConfirmChoice => {
                let key = self
                    .choice_list
                    .as_ref()
                    .map(|choice_list| choice_list.selected_key().clone())
                    .expect("Choice list must be present when selecting choice");

                self.runner.dispatch(RunnerAction::Select(key));
            }
        }
        None
    }

    fn handle_event(&mut self, event: tui::Event) -> Option<Action> {
        match event {
            tui::Event::Render => Some(Action::CheckView),
            tui::Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('f') => Some(Action::FastForward),
                KeyCode::Tab => Some(Action::SkipMessage),
                KeyCode::Enter => self
                    .choice_list
                    .is_some()
                    .then_some(Action::ConfirmChoice)
                    .or(Some(Action::NextLine)),
                k @ (KeyCode::Up | KeyCode::Down) => self.choice_list.as_ref().map(|_| match k {
                    KeyCode::Up => Action::SelectChoice(false),
                    KeyCode::Down => Action::SelectChoice(true),
                    _ => unreachable!(),
                }),
                _ => None,
            },
            _ => None,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
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
    CompleteMessage,
    NextLine,
    SelectChoice(bool),
    ConfirmChoice,
}
