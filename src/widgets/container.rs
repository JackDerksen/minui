//! # Container Widget
//!
//! A powerful layout management widget that automatically arranges and positions child
//! widgets within structured containers. The container system forms the backbone of
//! MinUI's layout capabilities, providing flexible arrangements with automatic sizing,
//! borders, padding, and alignment options.
//!
//! ## Features
//!
//! - **Automatic layout**: Intelligent child widget positioning and sizing
//! - **Dual layout modes**: Vertical stacking and horizontal arrangement
//! - **Flexible borders**: Optional visible borders with customizable styles
//! - **Advanced padding**: Per-side padding control with uniform options
//! - **Auto-sizing**: Containers adapt to content or fixed dimensions
//! - **Content alignment**: Normal positioning or automatic centering
//! - **Nested containers**: Support for complex hierarchical layouts
//!
//! ## Layout System
//!
//! Containers use a CSS Flexbox-inspired layout model:
//!
//! ```text
//! Vertical Layout:           Horizontal Layout:
//! ┌─────────────────┐       ┌─────┬─────┬─────┐
//! │    Widget 1     │       │ W1  │ W2  │ W3  │
//! ├─────────────────┤       │     │     │     │
//! │    Widget 2     │       │     │     │     │
//! ├─────────────────┤       └─────┴─────┴─────┘
//! │    Widget 3     │
//! └─────────────────┘
//! ```
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::{Container, Label, Text, LayoutDirection};
//!
//! // Create a vertical layout
//! let vertical_container = Container::vertical()
//!     .add_child(Label::new("Title"))
//!     .add_child(Text::new("Content goes here"))
//!     .add_child(Text::new("More content"));
//!
//! // Create a horizontal layout
//! let horizontal_container = Container::horizontal()
//!     .add_child(Text::new("Left"))
//!     .add_child(Text::new("Center"))
//!     .add_child(Text::new("Right"));
//! ```
//!
//! ## Styled Containers
//!
//! ```rust
//! use minui::{Container, BorderChars, Color, ColorPair, Padding};
//!
//! let styled_container = Container::new(LayoutDirection::Vertical)
//!     .with_border(BorderChars::double_line())
//!     .with_border_color(Some(ColorPair::new(Color::Blue, Color::Black)))
//!     .with_background_color(Some(ColorPair::new(Color::White, Color::Gray)))
//!     .with_padding(Padding::uniform(2))
//!     .with_content_alignment(ContentAlignment::AutoCenter);
//! ```
//!
//! ## Advanced Layouts
//!
//! ```rust
//! use minui::{Container, Panel, TextBlock, LayoutDirection, Padding};
//!
//! // Create a complex dashboard layout
//! let dashboard = Container::vertical()
//!     .with_padding(Padding::uniform(1))
//!     .add_child(
//!         Panel::new(60, 3)
//!             .with_header("System Dashboard")
//!             .with_body("Welcome to the monitoring interface")
//!     )
//!     .add_child(
//!         Container::horizontal()
//!             .add_child(
//!                 Panel::new(30, 10)
//!                     .with_header("CPU Usage")
//!                     .with_body("45% utilization")
//!             )
//!             .add_child(
//!                 Panel::new(30, 10)
//!                     .with_header("Memory")
//!                     .with_body("2.1GB / 8GB used")
//!             )
//!     );
//! ```
//!
//! ## Responsive Design
//!
//! ```rust
//! use minui::{Container, Text, LayoutDirection, ContentAlignment};
//!
//! // Container that adapts to available space
//! let responsive_layout = Container::new(LayoutDirection::Vertical)
//!     .with_auto_size(true)
//!     .with_content_alignment(ContentAlignment::AutoCenter)
//!     .add_child(Text::new("This content will be centered"))
//!     .add_child(Text::new("And this container will size to fit"));
//! ```
//!
//! ## Best Practices
//!
//! For deeply nested layouts, extract components into named variables or helper functions.
//! This improves readability and reusability:
//!
//! ```rust
//! use minui::{Container, Label, Text};
//!
//! let header = Container::vertical()
//!     .add_child(Label::new("Title"));
//!
//! let body = Container::vertical()
//!     .add_child(Text::new("Content"));
//!
//! let app = Container::vertical()
//!     .add_child(header)
//!     .add_child(body);
//! ```
//!
//! Container widgets form the foundation of MinUI's layout system, enabling the creation
//! of sophisticated terminal user interfaces with minimal code and maximum flexibility.

use super::{BorderChars, Widget};
use crate::widgets::common::WindowView;
use crate::{Color, ColorPair, Result, Window};

/// Layout direction for container children
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    /// Stack children vertically (like CSS flex-direction: column)
    Vertical,
    /// Arrange children horizontally (like CSS flex-direction: row)
    Horizontal,
}

/// Border visibility and styling options
#[derive(Debug, Clone)]
pub enum BorderStyle {
    /// No border - invisible container
    None,
    /// Visible border with specified character set
    Visible(BorderChars),
}

/// Padding configuration for containers
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

/// Content alignment within a container
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentAlignment {
    /// Content positioned normally within the container
    Normal,
    /// Content automatically centered within the container's full width
    AutoCenter,
}

impl Padding {
    /// Create uniform padding on all sides
    pub fn uniform(amount: u16) -> Self {
        Self {
            top: amount,
            right: amount,
            bottom: amount,
            left: amount,
        }
    }

    /// Create asymmetric padding (vertical, horizontal)
    pub fn symmetric(vertical: u16, horizontal: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create completely custom padding
    pub fn custom(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Total horizontal padding (left + right)
    pub fn horizontal_total(&self) -> u16 {
        self.left + self.right
    }

    /// Total vertical padding (top + bottom)
    pub fn vertical_total(&self) -> u16 {
        self.top + self.bottom
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::uniform(0)
    }
}

/// A layout-managed container widget that can hold multiple child widgets
pub struct Container {
    /// X-coordinate of the container position
    x: u16,
    /// Y-coordinate of the container position
    y: u16,
    /// Width of the container (including borders)
    width: u16,
    /// Height of the container (including borders)
    height: u16,
    /// Border styling
    border_style: BorderStyle,
    /// Optional border color
    border_color: Option<ColorPair>,
    /// Child widgets managed by this container
    children: Vec<Box<dyn Widget>>,
    /// Layout direction for children
    layout_direction: LayoutDirection,
    /// Internal padding
    padding: Padding,
    /// Spacing between child widgets
    child_spacing: u16,
    /// Whether to automatically size based on content
    auto_size: bool,
    /// Whether to take up full terminal window size
    fullscreen: bool,
    /// Content alignment within the container
    content_alignment: ContentAlignment,
}

impl Container {
    /// Creates a new container at the specified position
    ///
    /// # Arguments
    /// * `x` - X-coordinate position
    /// * `y` - Y-coordinate position
    /// * `width` - Container width (0 for auto-sizing)
    /// * `height` - Container height (0 for auto-sizing)
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
            border_style: BorderStyle::Visible(BorderChars::single_line()),
            border_color: None,
            children: Vec::new(),
            layout_direction: LayoutDirection::Vertical,
            padding: Padding::uniform(1),
            child_spacing: 0,
            auto_size: width == 0 || height == 0,
            fullscreen: false,
            content_alignment: ContentAlignment::Normal,
        }
    }

    /// Creates a fullscreen container that takes up the entire terminal
    pub fn fullscreen() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0, // Will be set by update_fullscreen_size
            height: 0,
            border_style: BorderStyle::None,
            border_color: None,
            children: Vec::new(),
            layout_direction: LayoutDirection::Vertical,
            padding: Padding::uniform(0),
            child_spacing: 0,
            auto_size: false,
            fullscreen: true,
            content_alignment: ContentAlignment::Normal,
        }
    }

    /// Creates a fullscreen container with the given terminal dimensions
    pub fn fullscreen_with_size(width: u16, height: u16) -> Self {
        let mut container = Self::fullscreen();
        container.update_fullscreen_size(width, height);
        container
    }

    /// Creates a vertical layout container (VBox equivalent)
    pub fn vertical() -> Self {
        Self::new(0, 0, 0, 0).with_layout_direction(LayoutDirection::Vertical)
    }

    /// Creates a horizontal layout container (HBox equivalent)
    pub fn horizontal() -> Self {
        Self::new(0, 0, 0, 0).with_layout_direction(LayoutDirection::Horizontal)
    }

    /// Sets the border style for the container
    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    /// Sets visible border with specified character set
    pub fn with_border(mut self, chars: BorderChars) -> Self {
        self.border_style = BorderStyle::Visible(chars);
        self
    }

    /// Removes the border (invisible container)
    pub fn without_border(mut self) -> Self {
        self.border_style = BorderStyle::None;
        self
    }

    /// Sets the border color
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(ColorPair::new(color, Color::Transparent));
        self
    }

    /// Sets the layout direction for children
    pub fn with_layout_direction(mut self, direction: LayoutDirection) -> Self {
        self.layout_direction = direction;
        self
    }

    /// Sets uniform padding on all sides
    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = Padding::uniform(padding);
        if self.auto_size {
            self.calculate_size();
        }
        self
    }

    /// Sets custom padding configuration
    pub fn with_padding_config(mut self, padding: Padding) -> Self {
        self.padding = padding;
        if self.auto_size {
            self.calculate_size();
        }
        self
    }

    /// Sets content to auto-center within the container
    pub fn with_auto_center(mut self) -> Self {
        self.content_alignment = ContentAlignment::AutoCenter;
        self
    }

    /// Sets spacing between child widgets
    pub fn with_child_spacing(mut self, spacing: u16) -> Self {
        self.child_spacing = spacing;
        if self.auto_size {
            self.calculate_size();
        }
        self
    }

    /// Enables or disables automatic sizing
    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        if auto_size {
            self.calculate_size();
        }
        self
    }

    /// Sets a fixed size for the container and disables auto-sizing
    ///
    /// # Arguments
    /// * `width` - Fixed width of the container
    /// * `height` - Fixed height of the container
    pub fn with_fixed_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self.auto_size = false;
        self
    }

    /// Adds a child widget to the container
    pub fn add_child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        if self.auto_size {
            self.calculate_size();
        }
        self
    }

    /// Adds multiple child widgets at once
    pub fn add_children(mut self, widgets: Vec<Box<dyn Widget>>) -> Self {
        self.children.extend(widgets);
        if self.auto_size {
            self.calculate_size();
        }
        self
    }

    /// Returns the content area size (width, height) available for child widgets
    ///
    /// This accounts for borders and padding, giving you the exact dimensions
    /// you should use when sizing child widgets for proper viewport constraint.
    ///
    /// # Example
    /// ```ignore
    /// let container = Container::new(0, 0, 60, 18);
    /// let (content_width, content_height) = container.get_content_size();
    /// let text_block = TextBlock::new(content_width, content_height, "...");
    /// ```
    pub fn get_content_size(&self) -> (u16, u16) {
        let border_thickness = self.border_thickness();
        let content_width = self
            .width
            .saturating_sub(border_thickness * 2)
            .saturating_sub(self.padding.horizontal_total());
        let content_height = self
            .height
            .saturating_sub(border_thickness * 2)
            .saturating_sub(self.padding.vertical_total());

        (content_width, content_height)
    }

    /// Returns the border thickness (0 for invisible borders)
    fn border_thickness(&self) -> u16 {
        match self.border_style {
            BorderStyle::None => 0,
            BorderStyle::Visible(_) => 1,
        }
    }

    /// Updates fullscreen container size to match terminal dimensions
    pub fn update_fullscreen_size(&mut self, terminal_width: u16, terminal_height: u16) {
        if self.fullscreen {
            self.width = terminal_width;
            self.height = terminal_height;
        }
    }

    /// Calculates the content area dimensions (excluding borders and padding)
    fn get_content_area(&self) -> (u16, u16, u16, u16) {
        let border_thickness = self.border_thickness();
        let content_x = self.x + border_thickness + self.padding.left;
        let content_y = self.y + border_thickness + self.padding.top;

        let content_width = self
            .width
            .saturating_sub(border_thickness * 2)
            .saturating_sub(self.padding.horizontal_total());
        let content_height = self
            .height
            .saturating_sub(border_thickness * 2)
            .saturating_sub(self.padding.vertical_total());

        (content_x, content_y, content_width, content_height)
    }

    /// Automatically calculates container size based on children
    fn calculate_size(&mut self) {
        if self.children.is_empty() {
            let border_thickness = self.border_thickness();
            self.width = border_thickness * 2 + self.padding.horizontal_total();
            self.height = border_thickness * 2 + self.padding.vertical_total();
            return;
        }

        let border_thickness = self.border_thickness();
        let mut required_width = 0u16;
        let mut required_height = 0u16;

        match self.layout_direction {
            LayoutDirection::Vertical => {
                // Width: maximum child width
                // Height: sum of all child heights + spacing
                for (i, child) in self.children.iter().enumerate() {
                    let (child_width, child_height) = child.get_size();
                    required_width = required_width.max(child_width);
                    required_height += child_height;

                    if i < self.children.len() - 1 {
                        required_height += self.child_spacing;
                    }
                }
            }
            LayoutDirection::Horizontal => {
                // Width: sum of all child widths + spacing
                // Height: maximum child height
                for (i, child) in self.children.iter().enumerate() {
                    let (child_width, child_height) = child.get_size();
                    required_width += child_width;
                    required_height = required_height.max(child_height);

                    if i < self.children.len() - 1 {
                        required_width += self.child_spacing;
                    }
                }
            }
        }

        self.width = required_width + (border_thickness * 2) + self.padding.horizontal_total();
        self.height = required_height + (border_thickness * 2) + self.padding.vertical_total();
    }

    /// Draws the border if it's visible
    fn draw_border(&self, window: &mut dyn Window) -> Result<()> {
        if let BorderStyle::Visible(border_chars) = &self.border_style {
            let mut draw_colored = |y: u16, x: u16, ch: char| -> Result<()> {
                if let Some(color) = self.border_color {
                    window.write_str_colored(y, x, &ch.to_string(), color)
                } else {
                    window.write_str(y, x, &ch.to_string())
                }
            };

            // Draw corners
            draw_colored(self.y, self.x, border_chars.top_left)?;
            draw_colored(self.y, self.x + self.width - 1, border_chars.top_right)?;
            draw_colored(self.y + self.height - 1, self.x, border_chars.bottom_left)?;
            draw_colored(
                self.y + self.height - 1,
                self.x + self.width - 1,
                border_chars.bottom_right,
            )?;

            // Draw horizontal borders
            for x in 1..self.width - 1 {
                draw_colored(self.y, self.x + x, border_chars.horizontal)?;
                draw_colored(
                    self.y + self.height - 1,
                    self.x + x,
                    border_chars.horizontal,
                )?;
            }

            // Draw vertical borders
            for y in 1..self.height - 1 {
                draw_colored(self.y + y, self.x, border_chars.vertical)?;
                draw_colored(self.y + y, self.x + self.width - 1, border_chars.vertical)?;
            }
        }
        Ok(())
    }

    /// Layouts and draws all child widgets
    fn draw_children(&self, window: &mut dyn Window) -> Result<()> {
        let (content_x, content_y, content_width, content_height) = self.get_content_area();

        // Calculate total content size for auto-centering
        let (total_content_width, total_content_height) =
            if matches!(self.content_alignment, ContentAlignment::AutoCenter) {
                self.calculate_total_content_size()
            } else {
                (0, 0) // Not needed for normal alignment
            };

        // Calculate centering offsets for the entire content block
        let (start_x, start_y) = match self.content_alignment {
            ContentAlignment::Normal => (0, 0),
            ContentAlignment::AutoCenter => {
                let offset_x = match self.layout_direction {
                    LayoutDirection::Vertical => {
                        // For vertical layout, center the widest child horizontally
                        if total_content_width < content_width {
                            (content_width - total_content_width) / 2
                        } else {
                            0
                        }
                    }
                    LayoutDirection::Horizontal => {
                        // For horizontal layout, center the entire content block
                        if total_content_width < content_width {
                            (content_width - total_content_width) / 2
                        } else {
                            0
                        }
                    }
                };
                let offset_y = if total_content_height < content_height {
                    (content_height - total_content_height) / 2
                } else {
                    0
                };
                (offset_x, offset_y)
            }
        };

        let mut current_x = start_x;
        let mut current_y = start_y;

        for (i, child) in self.children.iter().enumerate() {
            let (child_width, child_height) = child.get_size();

            // Calculate final child position
            let (child_x, child_y) = match self.content_alignment {
                ContentAlignment::Normal => (current_x, current_y),
                ContentAlignment::AutoCenter => {
                    match self.layout_direction {
                        LayoutDirection::Vertical => {
                            // For vertical layout in auto-center mode, center each child individually
                            // but ensure precise centering calculation
                            let centered_x = if child_width < content_width {
                                let available_space = content_width - child_width;
                                available_space / 2
                            } else {
                                0
                            };
                            (centered_x, current_y)
                        }
                        LayoutDirection::Horizontal => {
                            // For horizontal layout, use calculated positions
                            (current_x, start_y)
                        }
                    }
                }
            };

            // Create a window view for the child within our content area
            let mut child_window = WindowView {
                window,
                x_offset: content_x + child_x,
                y_offset: content_y + child_y,
                width: content_width.saturating_sub(child_x),
                height: content_height.saturating_sub(child_y),
            };

            // Draw the child widget
            child.draw(&mut child_window)?;

            // Update position for next child
            match self.layout_direction {
                LayoutDirection::Vertical => {
                    current_y += child_height;
                    if i < self.children.len() - 1 {
                        current_y += self.child_spacing;
                    }
                }
                LayoutDirection::Horizontal => {
                    current_x += child_width;
                    if i < self.children.len() - 1 {
                        current_x += self.child_spacing;
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculates the total size of all content for centering purposes
    fn calculate_total_content_size(&self) -> (u16, u16) {
        if self.children.is_empty() {
            return (0, 0);
        }

        let mut total_width = 0u16;
        let mut total_height = 0u16;

        match self.layout_direction {
            LayoutDirection::Vertical => {
                // Width: maximum child width
                // Height: sum of all child heights + spacing
                for (i, child) in self.children.iter().enumerate() {
                    let (child_width, child_height) = child.get_size();
                    total_width = total_width.max(child_width);
                    total_height += child_height;

                    if i < self.children.len() - 1 {
                        total_height += self.child_spacing;
                    }
                }
            }
            LayoutDirection::Horizontal => {
                // Width: sum of all child widths + spacing
                // Height: maximum child height
                for (i, child) in self.children.iter().enumerate() {
                    let (child_width, child_height) = child.get_size();
                    total_width += child_width;
                    total_height = total_height.max(child_height);

                    if i < self.children.len() - 1 {
                        total_width += self.child_spacing;
                    }
                }
            }
        }

        (total_width, total_height)
    }
}

impl Widget for Container {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        // For fullscreen containers, we assume they've been properly sized externally
        // before calling draw(). The draw() method shouldn't mutate the container.

        // Draw border first (if visible)
        self.draw_border(window)?;

        // Then draw children
        self.draw_children(window)?;

        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        // For fullscreen containers that haven't been sized yet, return terminal size
        if self.fullscreen && (self.width == 0 || self.height == 0) {
            // This is a fallback - ideally update_fullscreen_size should be called first
            (80, 24) // Default terminal size
        } else {
            (self.width, self.height)
        }
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

// Convenience constructors for common patterns
impl Container {
    /// Creates a simple div-like container with no border
    pub fn div() -> Self {
        Self::new(0, 0, 0, 0).without_border()
    }

    /// Creates a bordered panel-like container
    pub fn panel() -> Self {
        Self::new(0, 0, 0, 0)
            .with_border(BorderChars::single_line())
            .with_padding(1)
    }

    /// Creates a card-like container with double border
    pub fn card() -> Self {
        Self::new(0, 0, 0, 0)
            .with_border(BorderChars::double_line())
            .with_padding(2)
    }

    /// Creates a container that centers a single text widget horizontally
    /// This is a convenience method for the common pattern of centering text
    pub fn centered_text(text: impl Into<String>, terminal_width: u16) -> Self {
        use crate::widgets::Text;

        let text_string = text.into();
        let text_len = text_string.chars().count() as u16;
        let left_padding = if terminal_width > text_len {
            (terminal_width - text_len) / 2
        } else {
            0
        };

        Self::div()
            .add_child(Text::new(text_string))
            .with_padding_config(Padding::custom(0, 0, 0, left_padding))
    }

    /// Creates a container that centers colored text horizontally
    pub fn centered_colored_text(
        text: impl Into<String>,
        color: crate::Color,
        terminal_width: u16,
    ) -> Self {
        use crate::widgets::Text;

        let text_string = text.into();
        let text_len = text_string.chars().count() as u16;
        let left_padding = if terminal_width > text_len {
            (terminal_width - text_len) / 2
        } else {
            0
        };

        Self::div()
            .add_child(Text::new(text_string).with_text_color(color))
            .with_padding_config(Padding::custom(0, 0, 0, left_padding))
    }
}
