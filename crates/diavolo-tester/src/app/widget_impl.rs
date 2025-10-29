use super::App;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap};

impl Widget for &mut App<'_, '_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [dialogue_area, log_area, help_area] = Layout::vertical(
            [
                Constraint::Min(5),
                Constraint::Percentage(100),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .areas(area);

        let [message_area, choice_area] = Layout::horizontal(
            [
                Constraint::Percentage(100),
                Constraint::Min(
                    self.view_cache
                        .is_choice_available()
                        .then_some(20)
                        .unwrap_or(0),
                ),
            ]
            .as_ref(),
        )
        .areas(dialogue_area);

        self.render_log(log_area, buf);
        App::render_help(help_area, buf);

        if self.view_cache.has_message() {
            self.render_message(message_area, buf);
        }

        if self.view_cache.is_choice_available() {
            self.render_choice(choice_area, buf);
        }
    }
}

impl App<'_, '_> {
    fn render_message(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Left)
            .title("Message");

        let text = self.view_cache.message().unwrap_or_default();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
        Widget::render(paragraph.block(block), area, buf);
    }

    fn render_choice(&mut self, area: Rect, buf: &mut Buffer) {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::widgets::{List, ListItem};
        const SELECTED_STYLE: Style = Style::new().bg(Color::White).add_modifier(Modifier::BOLD);

        let block = Block::new()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Left)
            .title("Choice/Confirm");

        match self.choice_list.as_mut() {
            None => Widget::render(block, area, buf),
            Some(choice_list) => {
                let items = choice_list
                    .items
                    .iter()
                    .map(ListItem::from)
                    .collect::<Vec<_>>();
                let list = List::new(items).highlight_style(SELECTED_STYLE);
                StatefulWidget::render(list.block(block), area, buf, &mut choice_list.state)
            }
        };
    }

    fn render_log(&mut self, area: Rect, buf: &mut Buffer) {
        use ratatui::style::{Color, Style};
        use ratatui::text::{Line, Span, Text};

        let logs = self.log_collector.get_logs();
        let log_count = logs.len();
        if self.prev_log_count != log_count {
            self.prev_log_count = log_count;
            self.update_log_scroll();
        }

        let content_height = area.height.saturating_sub(2) as usize;

        let start_idx = if self.log_auto_scroll && log_count > content_height {
            log_count - content_height
        } else if self.log_scroll_offset + content_height >= log_count {
            log_count.saturating_sub(content_height)
        } else {
            self.log_scroll_offset
        };
        let end_idx = (start_idx + content_height).min(log_count);

        let log_lines = logs[start_idx..end_idx]
            .iter()
            .rev()
            .map(|entry| {
                use tracing::Level;
                let level_style = match entry.level {
                    Level::ERROR => Style::default().fg(Color::Red),
                    Level::WARN => Style::default().fg(Color::Yellow),
                    Level::INFO => Style::default().fg(Color::Blue),
                    Level::DEBUG => Style::default().fg(Color::Green),
                    Level::TRACE => Style::default().fg(Color::Magenta),
                };
                Line::from(vec![
                    Span::styled(format!("[{}] ", entry.level), level_style),
                    Span::raw(format!(
                        "{}: {}",
                        entry.timestamp.format("%H:%M:%S%.3f"),
                        entry.message
                    )),
                ])
            })
            .collect::<Vec<_>>();

        let text = Text::from(log_lines);

        let title = if self.log_auto_scroll {
            format!("Log Output ({}/{}) [Auto]", end_idx, log_count)
        } else {
            format!(
                "Log Output ({}/{}) [Manual - Line {}]",
                end_idx,
                log_count,
                start_idx + 1
            )
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Left)
            .title(title);

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

        Widget::render(paragraph.block(block), area, buf);
    }

    fn render_help(area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Left)
            .title("Help");
        let text = format!("Press 'q' to quit.").to_string().dark_gray();
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

        Widget::render(paragraph.block(block), area, buf);
    }
}
