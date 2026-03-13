// src/ui/components/searchable_dropdown.rs
use crate::models::AuthorModel;
use crate::ui::{BookFilter, Message as GlobalMessage};
use iced::border::width;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{padding, Element, Length, Task};

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    Search(String),
    Select(Option<usize>),
}

pub struct SearchableDropdown<T: PartialEq<String> + PartialEq<T>> {
    pub options: Vec<T>,
    selected: Option<usize>,
    search_term: String,
    is_open: bool,
    on_change_msg: Box<dyn Fn(T) -> GlobalMessage>,
    default_placeholder: String,
}

impl<T: AsRef<str> + Clone + PartialEq<String> + PartialEq<T>> SearchableDropdown<T> {
    pub fn new(
        options: Vec<T>,
        on_change_msg: Box<dyn Fn(T) -> GlobalMessage>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            options,
            selected,
            search_term: String::new(),
            is_open: false,
            on_change_msg,
            default_placeholder: String::from("Select"),
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if !self.is_open {
            self.search_term = String::new(); // Clear search when closing
        }
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.search_term = String::new();
    }

    pub fn search(&mut self, term: String) {
        self.search_term = term;
    }

    pub fn select(&mut self, index: usize) {
        self.selected = Some(index);
        self.close();
    }

    pub fn update(&mut self, msg: Message) -> Task<GlobalMessage> {
        match msg {
            Message::Toggle => {
                self.toggle();
                iced::Task::none()
            }
            Message::Select(index) => match index {
                None => iced::Task::none(),
                Some(idx) => {
                    self.select(idx);
                    let option = self.options.get(idx).unwrap();
                    iced::Task::done((self.on_change_msg)(option.clone()))
                }
            },
            Message::Search(text) => {
                self.search(text);
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<GlobalMessage> {
        // Filter options by search term
        let filtered_options = if self.search_term.is_empty() {
            self.options.clone()
        } else {
            self.options
                .iter()
                .filter(|option| {
                    option
                        .as_ref()
                        .to_lowercase()
                        .contains(&self.search_term.to_lowercase())
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        let selected_option = self.selected.and_then(|idx| self.options.get(idx));
        let placeholder: &str = selected_option
            .map(|s| s.as_ref())
            .unwrap_or(&self.default_placeholder);

        let header = button(text(placeholder).width(Length::Fill))
            .padding(10)
            .on_press(GlobalMessage::SearchableDropdownMessages(Message::Toggle))
            .width(Length::Fill)
            .style(button::secondary);

        if self.is_open {
            let search_input = text_input("Search author...", &self.search_term)
                .on_input(|text| GlobalMessage::SearchableDropdownMessages(Message::Search(text)))
                .padding(10)
                .width(Length::Fill);

            let options_list = if filtered_options.is_empty() {
                scrollable(container(text("No matching authors").size(14)).padding(10))
                    .height(Length::Fill)
                    .width(Length::Fill)
            } else {
                let options_column = column(filtered_options.iter().map(|(option)| {
                    let name = String::from(option.as_ref());

                    let is_selected = false;

                    let index = self.options.iter().position(|o| o == option);

                    container(
                        button(text(name).size(14))
                            .on_press(GlobalMessage::SearchableDropdownMessages(Message::Select(
                                index,
                            )))
                            .padding(8)
                            .width(Length::Fill)
                            .style(if is_selected {
                                button::primary
                            } else {
                                button::secondary
                            }),
                    )
                    .width(Length::Fill)
                    .into()
                }))
                .spacing(2)
                .width(Length::Fill);

                scrollable(options_column).height(200).width(Length::Fill)
            };

            column![header, search_input, options_list]
                .spacing(5)
                .width(Length::Fill)
                .into()
        } else {
            column![header].width(Length::Fill).into()
        }
    }
}
