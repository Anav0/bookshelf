// src/ui/components/searchable_dropdown.rs
use crate::models::AuthorModel;
use crate::ui::Message;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input,
};
use iced::{Element, Length};

// State for the searchable dropdown
#[derive(Debug, Clone)]
pub struct SearchableDropdown<T> {
    pub options: Vec<T>,
    selected: Option<T>,
    search_term: String,
    is_open: bool,
}

impl<T: Clone + PartialEq> SearchableDropdown<T> {
    pub fn new(options: Vec<T>, selected: Option<T>) -> Self {
        Self {
            options,
            selected,
            search_term: String::new(),
            is_open: false,
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

    pub fn select(&mut self, item: T) {
        self.selected = Some(item);
        self.close();
    }

    pub fn selected(&self) -> Option<&T> {
        self.selected.as_ref()
    }
}

// Implementation specific for AuthorModel
pub fn view_author_dropdown(
    dropdown: &SearchableDropdown<AuthorModel>,
    on_toggle: Message,
    on_search: impl Fn(String) -> Message + 'static,
    on_select: impl Fn(AuthorModel) -> Message + 'static,
) -> Element<Message> {
    // Filter options by search term
    let filtered_options = if dropdown.search_term.is_empty() {
        dropdown.options.clone()
    } else {
        dropdown
            .options
            .iter()
            .filter(|author| {
                let search_term = dropdown.search_term.to_lowercase();
                let author_name = author.Name.clone().unwrap_or_default().to_lowercase();

                author_name.contains(&search_term)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    // Create the dropdown header (either selected value or placeholder)
    let selected_text = dropdown
        .selected()
        .and_then(|author| author.Name.clone())
        .unwrap_or_else(|| "Select an author".to_string());

    let header = button(
        row![
            text(selected_text).width(Length::Fill),
            text(if dropdown.is_open { "▲" } else { "▼" })
        ]
        .spacing(10)
        .padding(5)
        .width(Length::Fill),
    )
    .on_press(on_toggle)
    .padding(10)
    .width(Length::Fill)
    .style(button::secondary);

    if dropdown.is_open {
        let search_input = text_input("Search author...", &dropdown.search_term)
            .on_input(on_search)
            .padding(10)
            .width(Length::Fill);

        let options_list = if filtered_options.is_empty() {
            scrollable(
                container(text("No matching authors").size(14))
                    .padding(10)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .height(Length::Fill)
            .width(Length::Fill)
        } else {
            let options_column = column(filtered_options.iter().map(|author| {
                let name = author
                    .Name
                    .clone()
                    .unwrap_or_else(|| "Unnamed Author".to_string());
                // Compare by ID for equality since we can't directly compare AuthorModel types
                let is_selected = dropdown
                    .selected()
                    .map(|selected_author| selected_author.Id == author.Id)
                    .unwrap_or(false);

                container(
                    button(text(name).size(14))
                        .on_press(on_select(author.clone()))
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
