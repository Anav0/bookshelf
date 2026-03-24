// src/ui/components/searchable_dropdown.rs
use crate::ui::Message as GlobalMessage;
use iced::widget::{button, combo_box, container, row, value};
use iced::{Element, Length, Task};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message<T: Clone> {
    Selected(T),
    Cleared,
}

pub struct SearchableDropdown<T: Display + Clone + 'static> {
    state: combo_box::State<T>,
    selected: Option<T>,
    on_select: Arc<dyn Fn(T) -> GlobalMessage>,

    original_placeholder: String,
    placeholder: String,
}

impl<T: Display + Clone + 'static> SearchableDropdown<T> {
    pub fn new(
        options: Vec<T>,
        placeholder: impl Into<String>,
        on_select: impl Fn(T) -> GlobalMessage + 'static,
    ) -> Self {
        let original_placeholder = placeholder.into().clone();
        let placeholder = original_placeholder.clone();
        Self {
            state: combo_box::State::new(options),
            selected: None,
            placeholder,
            original_placeholder,
            on_select: Arc::new(on_select),
        }
    }

    pub fn set_selected(&mut self, selected: T) {
        self.selected = Some(selected.clone());
        self.placeholder = format!("{}", selected);
    }

    pub fn clear(&mut self) {
        self.selected = None;
        self.placeholder = self.original_placeholder.clone();
    }

    pub fn view(&self) -> Element<'_, Message<T>> {
        row![
            combo_box(
                &self.state,
                &self.placeholder,
                self.selected.as_ref(),
                Message::Selected,
            ),
            button("clear")
                .on_press(Message::Cleared)
                .style(button::primary)
        ]
        .spacing(8)
        .into()
    }
}
