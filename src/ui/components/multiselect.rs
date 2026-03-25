use crate::ui::Message;
use iced::widget::{pick_list, row, text};
use iced::{Alignment, Element};
use std::fmt::{Display, Formatter};
use chrono::NaiveDate;

pub fn select<'b, T>(
    options: &'b [T],
    selected: Option<T>,
    label: &'b str,
    message_fn: impl Fn(T) -> Message + 'static,
) -> Element<'b, Message>
where
    T: Clone + Display + PartialEq + Eq,
{
    row![
        text(label).size(16),
        pick_list(options, selected, message_fn)
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}