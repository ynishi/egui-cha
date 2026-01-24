//! Chat - Simple chat component
//!
//! A scrollable chat view with message input.
//!
//! # Example
//!
//! ```ignore
//! // State initialization (in Model)
//! let mut chat_state = ChatState::new();
//!
//! // Adding messages
//! chat_state.push_user("Hello!");
//! chat_state.push_assistant("Hi! How can I help?");
//!
//! // Rendering with callback
//! if let Some(msg) = Chat::new(&mut chat_state).show(ui) {
//!     // User submitted a message
//!     println!("User sent: {}", msg);
//! }
//! ```

use crate::atoms::Button;
use crate::Theme;
use egui::{RichText, ScrollArea, Ui};
use std::collections::VecDeque;
use std::time::Instant;

/// Message sender role
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatRole {
    /// User message
    User,
    /// Assistant/Bot message
    Assistant,
    /// System message (centered, muted)
    System,
}

impl ChatRole {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::User => "You",
            Self::Assistant => "Assistant",
            Self::System => "System",
        }
    }
}

/// A single chat message
#[derive(Clone, Debug)]
pub struct ChatMessage {
    /// Message role/sender
    pub role: ChatRole,
    /// Message content
    pub content: String,
    /// Timestamp when created
    pub timestamp: Instant,
    /// Optional custom sender name (overrides role label)
    pub sender_name: Option<String>,
}

impl ChatMessage {
    /// Create a new message
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Instant::now(),
            sender_name: None,
        }
    }

    /// Set custom sender name
    pub fn with_sender(mut self, name: impl Into<String>) -> Self {
        self.sender_name = Some(name.into());
        self
    }

    /// Get display name for sender
    pub fn sender_display(&self) -> &str {
        self.sender_name.as_deref().unwrap_or(self.role.label())
    }
}

/// State for Chat (owned by parent)
pub struct ChatState {
    messages: VecDeque<ChatMessage>,
    max_messages: usize,
    /// Input text buffer
    pub input_text: String,
    /// Auto-scroll to bottom on new messages
    pub auto_scroll: bool,
    /// Track if new messages were added
    scroll_to_bottom: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 500,
            input_text: String::new(),
            auto_scroll: true,
            scroll_to_bottom: false,
        }
    }

    /// Set maximum messages to keep
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// Push a message
    pub fn push(&mut self, message: ChatMessage) {
        self.messages.push_back(message);

        // Trim old messages
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }

        if self.auto_scroll {
            self.scroll_to_bottom = true;
        }
    }

    /// Push a user message
    pub fn push_user(&mut self, content: impl Into<String>) {
        self.push(ChatMessage::new(ChatRole::User, content));
    }

    /// Push an assistant message
    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.push(ChatMessage::new(ChatRole::Assistant, content));
    }

    /// Push a system message
    pub fn push_system(&mut self, content: impl Into<String>) {
        self.push(ChatMessage::new(ChatRole::System, content));
    }

    /// Push with custom sender name
    pub fn push_from(
        &mut self,
        role: ChatRole,
        sender: impl Into<String>,
        content: impl Into<String>,
    ) {
        self.push(ChatMessage::new(role, content).with_sender(sender));
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Iterate messages
    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.iter()
    }

    /// Check and consume scroll flag
    fn take_scroll_flag(&mut self) -> bool {
        std::mem::take(&mut self.scroll_to_bottom)
    }
}

/// Chat component - displays a scrollable chat view with input
pub struct Chat<'a> {
    state: &'a mut ChatState,
    height: Option<f32>,
    show_input: bool,
    show_timestamp: bool,
    placeholder: &'a str,
    submit_label: &'a str,
    /// Custom color for system messages (default: theme.text_muted)
    system_message_color: Option<egui::Color32>,
}

impl<'a> Chat<'a> {
    /// Create a new Chat viewer
    pub fn new(state: &'a mut ChatState) -> Self {
        Self {
            state,
            height: None,
            show_input: true,
            show_timestamp: false,
            placeholder: "Type a message...",
            submit_label: "Send",
            system_message_color: None,
        }
    }

    /// Set fixed height (None = fill available space)
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }

    /// Show/hide input area
    pub fn show_input(mut self, show: bool) -> Self {
        self.show_input = show;
        self
    }

    /// Show/hide timestamps
    pub fn show_timestamp(mut self, show: bool) -> Self {
        self.show_timestamp = show;
        self
    }

    /// Set placeholder text for input
    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = text;
        self
    }

    /// Set submit button label
    pub fn submit_label(mut self, label: &'a str) -> Self {
        self.submit_label = label;
        self
    }

    /// Set custom color for system messages (default: theme.text_muted)
    pub fn system_message_color(mut self, color: egui::Color32) -> Self {
        self.system_message_color = Some(color);
        self
    }

    /// Show the chat and return submitted message (if any)
    pub fn show(mut self, ui: &mut Ui) -> Option<String> {
        let theme = Theme::current(ui.ctx());
        let mut submitted: Option<String> = None;

        ui.vertical(|ui| {
            // Message area
            self.render_messages(ui, &theme);

            // Input area
            if self.show_input {
                ui.add_space(theme.spacing_sm);
                submitted = self.render_input(ui, &theme);
            }
        });

        // Trigger scroll to bottom on next frame when message is submitted
        if submitted.is_some() {
            self.state.scroll_to_bottom = true;
        }

        submitted
    }

    fn render_messages(&mut self, ui: &mut Ui, theme: &Theme) {
        let scroll_to_bottom = self.state.take_scroll_flag();

        let scroll_area = if let Some(h) = self.height {
            ScrollArea::vertical().max_height(h)
        } else {
            ScrollArea::vertical()
        };

        scroll_area
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if self.state.is_empty() {
                    ui.label(
                        RichText::new("No messages yet")
                            .italics()
                            .color(theme.text_muted),
                    );
                } else {
                    let now = Instant::now();
                    let msg_count = self.state.messages.len();
                    for (i, message) in self.state.messages.iter().enumerate() {
                        self.render_message(ui, message, theme, now);
                        // Add spacing between messages (not after the last one)
                        if i < msg_count - 1 {
                            ui.add_space(theme.spacing_xs);
                        }
                    }
                }

                // Invisible anchor at bottom for scrolling
                let response = ui.allocate_response(egui::vec2(0.0, 0.0), egui::Sense::hover());
                if scroll_to_bottom {
                    response.scroll_to_me(Some(egui::Align::BOTTOM));
                }
            });
    }

    fn render_message(&self, ui: &mut Ui, message: &ChatMessage, theme: &Theme, now: Instant) {
        match message.role {
            ChatRole::System => {
                // System messages are centered
                let system_color = self.system_message_color.unwrap_or(theme.text_muted);
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() * 0.1);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&message.content)
                                .italics()
                                .color(system_color)
                                .size(theme.font_size_sm),
                        );
                    });
                });
            }
            _ => {
                // User/Assistant messages
                let is_user = message.role == ChatRole::User;
                let bubble_color = if is_user {
                    theme.primary
                } else {
                    theme.bg_secondary
                };
                let text_color = if is_user {
                    theme.primary_text
                } else {
                    theme.text_primary
                };

                let max_bubble_width = ui.available_width() * 0.7;

                ui.horizontal(|ui| {
                    // Left spacing for user messages (right-align)
                    if is_user {
                        ui.add_space(ui.available_width() - max_bubble_width);
                    }

                    egui::Frame::none()
                        .fill(bubble_color)
                        .inner_margin(theme.spacing_sm)
                        .corner_radius(theme.radius_md)
                        .show(ui, |ui| {
                            ui.set_max_width(max_bubble_width);

                            // Sender name (only for assistant)
                            if !is_user {
                                ui.label(
                                    RichText::new(message.sender_display())
                                        .strong()
                                        .size(theme.font_size_sm)
                                        .color(theme.text_secondary),
                                );
                            }

                            // Content
                            ui.label(RichText::new(&message.content).color(text_color));

                            // Timestamp (inline, right side)
                            if self.show_timestamp {
                                let elapsed = now.duration_since(message.timestamp);
                                let ts = if elapsed.as_secs() < 60 {
                                    "just now".to_string()
                                } else if elapsed.as_secs() < 3600 {
                                    format!("{}m ago", elapsed.as_secs() / 60)
                                } else {
                                    format!("{}h ago", elapsed.as_secs() / 3600)
                                };
                                ui.horizontal(|ui| {
                                    ui.add_space((ui.available_width() - 60.0).max(0.0));
                                    ui.label(RichText::new(ts).size(theme.font_size_xs).color(
                                        if is_user {
                                            theme.primary_text.gamma_multiply(0.7)
                                        } else {
                                            theme.text_muted
                                        },
                                    ));
                                });
                            }
                        });
                });
            }
        }
    }

    fn render_input(&mut self, ui: &mut Ui, theme: &Theme) -> Option<String> {
        let mut submitted = None;

        // Check for Enter without Shift (submit) before rendering
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

        ui.horizontal(|ui| {
            // Text input with custom frame
            let input_width = (ui.available_width() - 80.0).max(100.0);
            let input_response = egui::Frame::none()
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(theme.radius_sm)
                .fill(theme.bg_primary)
                .inner_margin(egui::Margin::symmetric(
                    theme.spacing_sm as i8,
                    theme.spacing_xs as i8,
                ))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.state.input_text)
                            .hint_text(self.placeholder)
                            .desired_width((input_width - 20.0).max(50.0))
                            .desired_rows(1)
                            .frame(false),
                    )
                });
            let response = input_response.inner;

            // Send button - match input height
            let can_send = !self.state.input_text.trim().is_empty();
            let send_clicked = ui
                .add_enabled(can_send, Button::primary(self.submit_label))
                .clicked();

            // Submit on Enter (without Shift) or button click
            if can_send && response.has_focus() && enter_pressed {
                // Remove trailing newline that Enter adds
                self.state.input_text = self.state.input_text.trim_end().to_string();
                submitted = Some(std::mem::take(&mut self.state.input_text));
            } else if can_send && send_clicked {
                submitted = Some(std::mem::take(&mut self.state.input_text));
            }
        });

        submitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::new(ChatRole::User, "Hello").with_sender("Alice");

        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.sender_display(), "Alice");
    }

    #[test]
    fn test_chat_state_push() {
        let mut state = ChatState::new().with_max_messages(5);

        for i in 0..10 {
            state.push_user(format!("Message {}", i));
        }

        // Should only keep last 5
        assert_eq!(state.len(), 5);
    }

    #[test]
    fn test_chat_state_roles() {
        let mut state = ChatState::new();
        state.push_user("Hi");
        state.push_assistant("Hello!");
        state.push_system("User joined");

        assert_eq!(state.len(), 3);

        let msgs: Vec<_> = state.messages().collect();
        assert_eq!(msgs[0].role, ChatRole::User);
        assert_eq!(msgs[1].role, ChatRole::Assistant);
        assert_eq!(msgs[2].role, ChatRole::System);
    }

    #[test]
    fn test_role_labels() {
        assert_eq!(ChatRole::User.label(), "You");
        assert_eq!(ChatRole::Assistant.label(), "Assistant");
        assert_eq!(ChatRole::System.label(), "System");
    }
}
