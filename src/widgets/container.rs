//! # Container Widget
//!
//! A unified layout-managed container widget inspired by OpenTUI's "box" component.
//!
//! This widget is intended to be the primary building block for layout and styling in MinUI.
//! It supports borders, titles, background fills, padding, and arranging child widgets in
//! vertical or horizontal directions.
//!
//! ## Notes for future self:
//!
//! Some higher-level widgets (e.g. `ScrollBox`) may need to infer content sizing from a
//! container’s children. For that reason, `Container` exposes a small amount of read-only
//! layout metadata such as the computed content area and child list.
//!
//! ## Features
//!
//! - **Unified API**: One primary container for both layout and styling
//! - **Fine-grained borders**: Draw selective sides (top, bottom, left, right)
//! - **Built-in titles**: First-class title support with alignment options
//! - **Modern gaps**: Row and column spacing between children
//! - **Flexible styling**: Colors, custom border characters, background colors
//! - **Layout management**: Horizontal and vertical arrangements with auto-sizing
//! - **Focus states**: Separate border colors for normal and focused states
//!
//! ## Basic Usage
//!
//! ```rust
//! use minui::prelude::*;
//!
//! // Simple vertical container with title
//! let container = Container::vertical()
//!     .with_title("Welcome")
//!     .with_title_alignment(TitleAlignment::Center)
//!     .add_child(Label::new("Hello, World!"));
//! ```
//!
//! ## Selective Borders
//!
//! ```rust
//! use minui::prelude::*;
//!
//! // Only draw top and bottom borders
//! let divider = Container::new()
//!     .with_border_sides(vec![BorderSide::Top, BorderSide::Bottom])
//!     .with_title("Section");
//! ```
//!
//! ## Modern Gaps
//!
//! ```rust
//! use minui::prelude::*;
//!
//! let form = Container::vertical()
//!     .with_row_gap(Gap::Pixels(1))
//!     .add_child(Label::new("Name:"))
//!     .add_child(TextInput::new(20));
//! ```

use super::{BorderChars, Widget};
use crate::widgets::common::WindowView;
use crate::{Color, ColorPair, Result, Window};
use std::collections::HashSet;

/// Border side for selective border rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorderSide {
    /// Top border
    Top,
    /// Right border
    Right,
    /// Bottom border
    Bottom,
    /// Left border
    Left,
}

/// Title alignment within the box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAlignment {
    /// Align title to the left
    Left,
    /// Center the title
    Center,
    /// Align title to the right
    Right,
}

/// Gap size between children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    /// Fixed pixel gap
    Pixels(u16),
    /// Percentage-based gap (0-100)
    Percent(u8),
}

/// Border configuration for the box
#[derive(Debug, Clone)]
pub struct BorderConfig {
    /// Which sides to draw
    pub sides: HashSet<BorderSide>,
    /// Border character style
    pub chars: BorderChars,
    /// Normal border color
    pub color: ColorPair,
    /// Border color when focused
    pub focused_color: Option<ColorPair>,
}

impl BorderConfig {
    /// Creates a new border configuration with all sides enabled
    ///
    /// Note: currently unused in the codebase, but kept as a convenience constructor for
    /// future higher-level presets/helpers and user code that wants an explicit “all sides”
    /// config without calling `with_border()`.
    #[allow(dead_code)]
    pub fn all_sides(chars: BorderChars, color: ColorPair) -> Self {
        let mut sides = HashSet::new();
        sides.insert(BorderSide::Top);
        sides.insert(BorderSide::Right);
        sides.insert(BorderSide::Bottom);
        sides.insert(BorderSide::Left);

        Self {
            sides,
            chars,
            color,
            focused_color: None,
        }
    }

    /// Creates a new border configuration with no sides
    pub fn no_sides(chars: BorderChars, color: ColorPair) -> Self {
        Self {
            sides: HashSet::new(),
            chars,
            color,
            focused_color: None,
        }
    }

    /// Checks if any border sides are enabled
    pub fn has_border(&self) -> bool {
        !self.sides.is_empty()
    }
}

/// Layout direction for arranging children
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// Stack children vertically
    Vertical,
    /// Arrange children horizontally
    Horizontal,
}

/// Content alignment within the box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentAlignment {
    /// Content positioned normally
    Normal,
    /// Content automatically centered
    AutoCenter,
}

/// Padding configuration for the box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    /// Top padding
    pub top: u16,
    /// Right padding
    pub right: u16,
    /// Bottom padding
    pub bottom: u16,
    /// Left padding
    pub left: u16,
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

    /// Total horizontal padding
    pub fn horizontal_total(&self) -> u16 {
        self.left + self.right
    }

    /// Total vertical padding
    pub fn vertical_total(&self) -> u16 {
        self.top + self.bottom
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::uniform(0)
    }
}

/// A simple sizing spec for container children.
///
/// This is intentionally minimal for the first pass:
/// - `Auto`: use the child's intrinsic size along that axis.
/// - `Fixed`: force a fixed size.
/// - `Fill`: participate in fill sizing (even-split for the main axis, fill remaining on cross-axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeSpec {
    Auto,
    Fixed(u16),
    Fill,
}

impl Default for SizeSpec {
    fn default() -> Self {
        SizeSpec::Auto
    }
}

/// Optional min/max constraints for a single axis.
///
/// `max = None` means "unbounded".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisConstraints {
    pub min: u16,
    pub max: Option<u16>,
}

impl Default for AxisConstraints {
    fn default() -> Self {
        Self { min: 0, max: None }
    }
}

impl AxisConstraints {
    pub const fn new(min: u16, max: Option<u16>) -> Self {
        Self { min, max }
    }

    pub fn clamp(&self, v: u16) -> u16 {
        let v = v.max(self.min);
        match self.max {
            Some(max) => v.min(max),
            None => v,
        }
    }
}

/// Per-child constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChildConstraints {
    pub width: AxisConstraints,
    pub height: AxisConstraints,
}

struct ContainerChild {
    widget: Box<dyn Widget>,
    width: SizeSpec,
    height: SizeSpec,
    constraints: ChildConstraints,

    // Cached intrinsic size from time of insertion (used for Auto sizing).
    // A future layout pass may refresh this each frame if needed.
    intrinsic_width: u16,
    intrinsic_height: u16,
}

/// A unified layout-managed container widget
pub struct Container {
    /// X-coordinate position
    x: u16,
    /// Y-coordinate position
    y: u16,
    /// Width (including borders)
    width: u16,
    /// Height (including borders)
    height: u16,

    /// Layout direction
    layout_direction: LayoutDirection,
    /// Internal padding
    padding: Padding,
    /// Gap between children (applies to both directions if not overridden)
    gap: Option<Gap>,
    /// Row-specific gap
    row_gap: Option<Gap>,
    /// Column-specific gap
    column_gap: Option<Gap>,
    /// Content alignment
    content_alignment: ContentAlignment,
    /// Auto-sizing enabled
    auto_size: bool,
    /// Fullscreen mode
    fullscreen: bool,

    /// Border configuration
    border: BorderConfig,
    /// Background color
    background_color: Option<ColorPair>,
    /// Whether to fill background
    should_fill: bool,

    /// Title text
    title: Option<String>,
    /// Title alignment
    title_alignment: TitleAlignment,

    /// Focus state
    focused: bool,

    /// Child widgets.
    ///
    /// Children are stored with per-child sizing so `Container` can resolve layout
    /// deterministically within a known content rect (phase-2 sizing work).
    children: Vec<ContainerChild>,
}

impl Container {
    /// Creates a new container with default settings
    ///
    /// Defaults are intentionally "layout-first":
    /// - no borders
    /// - no background fill
    /// - zero padding
    /// - auto-sizing enabled
    pub fn new() -> Self {
        let mut this = Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            layout_direction: LayoutDirection::Vertical,
            padding: Padding::uniform(0),
            gap: None,
            row_gap: None,
            column_gap: None,
            content_alignment: ContentAlignment::Normal,
            auto_size: true,
            fullscreen: false,
            border: BorderConfig::no_sides(
                BorderChars::single_line(),
                ColorPair::new(Color::White, Color::Black),
            ),
            background_color: None,
            should_fill: false,
            title: None,
            title_alignment: TitleAlignment::Left,
            focused: false,
            children: Vec::new(),
        };

        this.recalculate_size();
        this
    }

    /// Creates a vertical container
    pub fn vertical() -> Self {
        Self::new().with_layout_direction(LayoutDirection::Vertical)
    }

    /// Creates a horizontal container
    pub fn horizontal() -> Self {
        Self::new().with_layout_direction(LayoutDirection::Horizontal)
    }

    /// Creates a fullscreen container
    pub fn fullscreen() -> Self {
        Self {
            fullscreen: true,
            auto_size: false,
            padding: Padding::uniform(0),
            ..Self::new()
        }
    }

    // Builder methods

    /// Sets the position and size
    pub fn with_position_and_size(mut self, x: u16, y: u16, width: u16, height: u16) -> Self {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.auto_size = false;
        self
    }

    /// Sets the layout direction
    pub fn with_layout_direction(mut self, direction: LayoutDirection) -> Self {
        self.layout_direction = direction;
        self.recalculate_size();
        self
    }

    /// Sets padding on all sides
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self.recalculate_size();
        self
    }

    /// Sets gap between all children
    pub fn with_gap(mut self, gap: Gap) -> Self {
        self.gap = Some(gap);
        self.recalculate_size();
        self
    }

    /// Sets gap between rows (vertical layout)
    pub fn with_row_gap(mut self, gap: Gap) -> Self {
        self.row_gap = Some(gap);
        self.recalculate_size();
        self
    }

    /// Sets gap between columns (horizontal layout)
    pub fn with_column_gap(mut self, gap: Gap) -> Self {
        self.column_gap = Some(gap);
        self.recalculate_size();
        self
    }

    /// Sets content alignment
    pub fn with_content_alignment(mut self, alignment: ContentAlignment) -> Self {
        self.content_alignment = alignment;
        self
    }

    /// Sets border sides to display
    pub fn with_border_sides(mut self, sides: Vec<BorderSide>) -> Self {
        self.border.sides = sides.into_iter().collect();
        self.recalculate_size();
        self
    }

    /// Enables all border sides
    pub fn with_border(mut self) -> Self {
        let mut sides = HashSet::new();
        sides.insert(BorderSide::Top);
        sides.insert(BorderSide::Right);
        sides.insert(BorderSide::Bottom);
        sides.insert(BorderSide::Left);
        self.border.sides = sides;
        self.recalculate_size();
        self
    }

    /// Disables all borders
    pub fn without_border(mut self) -> Self {
        self.border.sides.clear();
        self.recalculate_size();
        self
    }

    /// Sets border characters
    pub fn with_border_chars(mut self, chars: BorderChars) -> Self {
        self.border.chars = chars;
        // Smart initialization: enable all sides if we're setting chars
        if self.border.sides.is_empty() {
            self = self.with_border();
        }
        self.recalculate_size();
        self
    }

    /// Sets border color
    pub fn with_border_color(mut self, color: ColorPair) -> Self {
        self.border.color = color;
        // Smart initialization: enable all sides if we're setting color
        if self.border.sides.is_empty() {
            self = self.with_border();
        }
        self.recalculate_size();
        self
    }

    /// Sets focused border color
    pub fn with_focused_border_color(mut self, color: ColorPair) -> Self {
        self.border.focused_color = Some(color);
        // Smart initialization: enable all sides if we're setting color
        if self.border.sides.is_empty() {
            self = self.with_border();
        }
        self.recalculate_size();
        self
    }

    /// Sets title text
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        // Smart initialization: enable top border for title
        self.border.sides.insert(BorderSide::Top);
        self.recalculate_size();
        self
    }

    /// Sets title alignment
    pub fn with_title_alignment(mut self, alignment: TitleAlignment) -> Self {
        self.title_alignment = alignment;
        self
    }

    /// Sets background color
    pub fn with_background_color(mut self, color: ColorPair) -> Self {
        self.background_color = Some(color);
        self.should_fill = true;
        self
    }

    /// Sets whether to fill the background
    pub fn with_fill(mut self, fill: bool) -> Self {
        self.should_fill = fill;
        self
    }

    /// Adds a child widget.
    ///
    /// Default sizing:
    /// - Vertical layout: children fill width, auto height.
    /// - Horizontal layout: children auto width, fill height.
    pub fn add_child(mut self, child: impl Widget + 'static) -> Self {
        let (w, h) = child.get_size();

        let (width, height) = match self.layout_direction {
            LayoutDirection::Vertical => (SizeSpec::Fill, SizeSpec::Auto),
            LayoutDirection::Horizontal => (SizeSpec::Auto, SizeSpec::Fill),
        };

        self.children.push(ContainerChild {
            widget: Box::new(child),
            width,
            height,
            constraints: ChildConstraints::default(),
            // Cache current intrinsic size so auto-size and simple layout can work without a full pass.
            intrinsic_width: w,
            intrinsic_height: h,
        });

        self.recalculate_size();
        self
    }

    /// Adds a child that participates in **Fill** sizing on the container's main axis.
    ///
    /// - Vertical layout: `height = Fill` (remaining height is split evenly across Fill children)
    /// - Horizontal layout: `width = Fill` (remaining width is split evenly across Fill children)
    ///
    /// Cross-axis defaults remain consistent with `add_child`:
    /// - Vertical layout: `width = Fill`
    /// - Horizontal layout: `height = Fill`
    pub fn add_child_fill(mut self, child: impl Widget + 'static) -> Self {
        let (w, h) = child.get_size();

        let (width, height) = match self.layout_direction {
            LayoutDirection::Vertical => (SizeSpec::Fill, SizeSpec::Fill),
            LayoutDirection::Horizontal => (SizeSpec::Fill, SizeSpec::Fill),
        };

        self.children.push(ContainerChild {
            widget: Box::new(child),
            width,
            height,
            constraints: ChildConstraints::default(),
            intrinsic_width: w,
            intrinsic_height: h,
        });

        self.recalculate_size();
        self
    }

    /// Sets width constraints on the most recently added child.
    ///
    /// This is a minimal builder convenience for configuring per-child constraints without
    /// introducing a full retained layout API.
    pub fn last_child_width_constraints(mut self, min: u16, max: Option<u16>) -> Self {
        if let Some(last) = self.children.last_mut() {
            last.constraints.width = AxisConstraints::new(min, max);
        }
        self
    }

    /// Sets height constraints on the most recently added child.
    pub fn last_child_height_constraints(mut self, min: u16, max: Option<u16>) -> Self {
        if let Some(last) = self.children.last_mut() {
            last.constraints.height = AxisConstraints::new(min, max);
        }
        self
    }

    /// Adds a child with a fixed size on the container's **main axis**.
    ///
    /// - Vertical layout: `height = Fixed(main)`
    /// - Horizontal layout: `width = Fixed(main)`
    ///
    /// Cross-axis defaults remain consistent with `add_child`:
    /// - Vertical layout: `width = Fill`
    /// - Horizontal layout: `height = Fill`
    pub fn add_child_fixed_main(mut self, child: impl Widget + 'static, main: u16) -> Self {
        let (w, h) = child.get_size();

        let (width, height) = match self.layout_direction {
            LayoutDirection::Vertical => (SizeSpec::Fill, SizeSpec::Fixed(main)),
            LayoutDirection::Horizontal => (SizeSpec::Fixed(main), SizeSpec::Fill),
        };

        self.children.push(ContainerChild {
            widget: Box::new(child),
            width,
            height,
            constraints: ChildConstraints::default(),
            intrinsic_width: w,
            intrinsic_height: h,
        });

        self.recalculate_size();
        self
    }

    /// Returns a read-only view of this container's children.
    ///
    /// Returns borrowed references to child widgets (in insertion order).
    ///
    /// This is primarily intended for higher-level widgets (e.g. `ScrollBox`) that need to
    /// inspect children without taking ownership.
    ///
    /// Note: this returns references to the widgets, not the internal sizing wrappers.
    pub fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|c| c.widget.as_ref()).collect()
    }

    /// Returns this container's layout direction.
    ///
    /// This is primarily intended for higher-level widgets (e.g. `ScrollBox`) that need to
    /// infer content sizing from child intrinsic sizes.
    pub fn layout_direction(&self) -> LayoutDirection {
        self.layout_direction
    }

    // Accessors

    /// Returns the current focus state
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state
    pub fn set_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Returns the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Draws only the container "frame" (background fill + borders/title).
    ///
    /// This does **not** draw child widgets. This is useful for widgets like `ScrollBox`
    /// that want a static border while scrolling only the inner content.
    pub fn draw_frame(&self, window: &mut dyn Window) -> Result<()> {
        self.draw_background(window)?;
        self.draw_borders(window)?;
        Ok(())
    }

    /// Draws only the container contents (child widgets) into the provided window.
    ///
    /// This does **not** draw background fill or borders/title.
    /// This is useful for widgets like `ScrollBox` that want to clip/scroll content while
    /// keeping the container border static.
    pub fn draw_contents(&self, window: &mut dyn Window) -> Result<()> {
        self.draw_children(window)?;
        Ok(())
    }

    // Helper methods

    /// Gets the width of the left border if present
    fn border_left_width(&self) -> u16 {
        if self.border.sides.contains(&BorderSide::Left) {
            1
        } else {
            0
        }
    }

    /// Gets the width of the right border if present
    fn border_right_width(&self) -> u16 {
        if self.border.sides.contains(&BorderSide::Right) {
            1
        } else {
            0
        }
    }

    /// Gets the height of the top border if present
    fn border_top_height(&self) -> u16 {
        if self.border.sides.contains(&BorderSide::Top) {
            1
        } else {
            0
        }
    }

    /// Gets the height of the bottom border if present
    fn border_bottom_height(&self) -> u16 {
        if self.border.sides.contains(&BorderSide::Bottom) {
            1
        } else {
            0
        }
    }

    /// Gets the content area bounds as `(x, y, width, height)`.
    ///
    /// This is the rectangle inside borders and padding where child widgets are drawn.
    /// The coordinates are relative to the parent window.
    ///
    /// This is exposed publicly to support higher-level widgets (e.g. `ScrollBox`) that need
    /// to derive viewport/content sizing from the container layout.
    pub fn content_area(&self) -> (u16, u16, u16, u16) {
        let x = self.x + self.border_left_width() + self.padding.left;
        let y = self.y + self.border_top_height() + self.padding.top;
        let width = self.width.saturating_sub(
            self.border_left_width()
                + self.padding.left
                + self.padding.right
                + self.border_right_width(),
        );
        let height = self.height.saturating_sub(
            self.border_top_height()
                + self.padding.top
                + self.padding.bottom
                + self.border_bottom_height(),
        );
        (x, y, width, height)
    }

    /// Returns this container's padding configuration.
    ///
    /// This is intentionally a small, read-only accessor so higher-level widgets (and optional
    /// interaction registration helpers) can derive sub-areas without needing to cache geometry.
    pub fn padding(&self) -> Padding {
        self.padding
    }

    /// Returns this container's border configuration.
    ///
    /// This enables higher-level widgets (e.g. `ScrollBox`) to mirror `content_area()` math when
    /// given an explicit outer rect, without making the interaction system intrusive.
    pub fn border_config(&self) -> &BorderConfig {
        &self.border
    }

    /// Internal helper retained for backwards compatibility inside this module.
    fn get_content_area(&self) -> (u16, u16, u16, u16) {
        self.content_area()
    }

    /// Recomputes this container's size if `auto_size` is enabled.
    ///
    /// This uses children intrinsic sizes and converts them into an "outer" size by adding
    /// padding and border thickness. Percent gaps are treated as 1 cell (minimum) during
    /// auto-size to avoid circular "size depends on gap depends on size" behavior.
    fn recalculate_size(&mut self) {
        if !self.auto_size || self.fullscreen {
            return;
        }

        if self.children.is_empty() {
            // Still account for border + padding so empty containers can render a frame/title.
            let outer_w = self
                .border_left_width()
                .saturating_add(self.padding.left)
                .saturating_add(self.padding.right)
                .saturating_add(self.border_right_width());
            let outer_h = self
                .border_top_height()
                .saturating_add(self.padding.top)
                .saturating_add(self.padding.bottom)
                .saturating_add(self.border_bottom_height());

            self.width = outer_w;
            self.height = outer_h;
            return;
        }

        let mut content_required_w: u16 = 0;
        let mut content_required_h: u16 = 0;

        // Percent gaps can't be resolved during auto-size. Treat them as 1 cell to keep spacing
        // stable and avoid circular dependency.
        let gap_pixels: u16 = match self.layout_direction {
            LayoutDirection::Vertical => self.row_gap.or(self.gap),
            LayoutDirection::Horizontal => self.column_gap.or(self.gap),
        }
        .map(|g| match g {
            Gap::Pixels(n) => n,
            Gap::Percent(_) => 1,
        })
        .unwrap_or(0);

        match self.layout_direction {
            LayoutDirection::Vertical => {
                for (idx, child) in self.children.iter().enumerate() {
                    let (cw, ch) = (child.intrinsic_width, child.intrinsic_height);
                    content_required_w = content_required_w.max(cw);
                    content_required_h = content_required_h.saturating_add(ch);

                    if idx < self.children.len() - 1 {
                        content_required_h = content_required_h.saturating_add(gap_pixels);
                    }
                }
            }
            LayoutDirection::Horizontal => {
                for (idx, child) in self.children.iter().enumerate() {
                    let (cw, ch) = (child.intrinsic_width, child.intrinsic_height);
                    content_required_w = content_required_w.saturating_add(cw);
                    content_required_h = content_required_h.max(ch);

                    if idx < self.children.len() - 1 {
                        content_required_w = content_required_w.saturating_add(gap_pixels);
                    }
                }
            }
        }

        let outer_w = content_required_w
            .saturating_add(self.border_left_width())
            .saturating_add(self.padding.left)
            .saturating_add(self.padding.right)
            .saturating_add(self.border_right_width());

        let outer_h = content_required_h
            .saturating_add(self.border_top_height())
            .saturating_add(self.padding.top)
            .saturating_add(self.padding.bottom)
            .saturating_add(self.border_bottom_height());

        self.width = outer_w;
        self.height = outer_h;
    }

    /// Returns the "auto-size gap pixels" used by `recalculate_size()` for the current layout
    /// direction.
    ///
    /// This is intentionally *not* the fully resolved percent gap (which depends on available
    /// space). During auto-size (and in best-effort content measurement), percent gaps are treated
    /// as 1 cell to avoid circular dependencies.
    ///
    /// This is exposed publicly to support higher-level widgets (e.g. `ScrollBox`) that need to
    /// infer content sizing from child intrinsic sizes without re-running the full layout engine.
    pub fn autosize_gap_pixels(&self) -> u16 {
        let gap = match self.layout_direction {
            LayoutDirection::Vertical => self.row_gap.or(self.gap),
            LayoutDirection::Horizontal => self.column_gap.or(self.gap),
        };

        match gap {
            Some(Gap::Pixels(n)) => n,
            Some(Gap::Percent(_)) => 1,
            None => 0,
        }
    }

    /// Resolves the gap for the current layout direction
    fn resolve_gap(&self, available_space: u16) -> u16 {
        let gap = match self.layout_direction {
            LayoutDirection::Vertical => self.row_gap.or(self.gap),
            LayoutDirection::Horizontal => self.column_gap.or(self.gap),
        };

        match gap {
            Some(Gap::Pixels(n)) => n,
            Some(Gap::Percent(pct)) => {
                let percent = (pct as u16).min(100);
                (available_space * percent / 100).max(1)
            }
            None => 0,
        }
    }

    /// Draws the borders
    fn draw_borders(&self, window: &mut dyn Window) -> Result<()> {
        // If the container has no drawable area, skip borders entirely.
        //
        // Borders assume at least 1 cell of width/height for corners/lines; if size is 0
        // (or extremely small), border math can produce invalid coordinates.
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        if !self.border.has_border() {
            return Ok(());
        }

        let color = if self.focused {
            self.border.focused_color.unwrap_or(self.border.color)
        } else {
            self.border.color
        };

        let chars = self.border.chars;

        // Top border
        if self.border.sides.contains(&BorderSide::Top) {
            self.draw_top_border(window, chars, color)?;
        }

        // Bottom border
        if self.border.sides.contains(&BorderSide::Bottom) {
            self.draw_bottom_border(window, chars, color)?;
        }

        // Left border
        if self.border.sides.contains(&BorderSide::Left) {
            self.draw_left_border(window, chars, color)?;
        }

        // Right border
        if self.border.sides.contains(&BorderSide::Right) {
            self.draw_right_border(window, chars, color)?;
        }

        Ok(())
    }

    /// Draws the top border with optional title
    fn draw_top_border(
        &self,
        window: &mut dyn Window,
        chars: BorderChars,
        color: ColorPair,
    ) -> Result<()> {
        let has_left = self.border.sides.contains(&BorderSide::Left);
        let has_right = self.border.sides.contains(&BorderSide::Right);

        let mut x = self.x;

        // Draw left corner if present
        if has_left {
            window.write_str_colored(self.y, x, &chars.top_left.to_string(), color)?;
            x += 1;
        }

        // Calculate available space for title or line
        let available_width = self
            .width
            .saturating_sub(if has_left { 1 } else { 0 })
            .saturating_sub(if has_right { 1 } else { 0 });

        if let Some(title) = &self.title {
            self.draw_title_in_border(window, title, x, available_width, chars, color)?;
        } else {
            // Just draw a horizontal line
            for _ in 0..available_width {
                window.write_str_colored(self.y, x, &chars.horizontal.to_string(), color)?;
                x += 1;
            }
        }

        // Draw right corner if present
        if has_right {
            window.write_str_colored(
                self.y,
                self.x + self.width - 1,
                &chars.top_right.to_string(),
                color,
            )?;
        }

        Ok(())
    }

    /// Draws the title within the top border
    fn draw_title_in_border(
        &self,
        window: &mut dyn Window,
        title: &str,
        start_x: u16,
        available_width: u16,
        chars: BorderChars,
        color: ColorPair,
    ) -> Result<()> {
        // Clip the title by terminal *cell width*, not byte length, to avoid corrupting the border
        // when the terminal is narrow or when the title contains multi-byte characters.
        //
        // This also avoids slicing `&str` at non-UTF8 boundaries.
        let title_max_width = available_width.saturating_sub(2); // Space before and after
        let display_title_owned =
            crate::text::clip_to_cells(title, title_max_width, crate::text::TabPolicy::SingleCell);
        let display_title = display_title_owned.as_str();

        let title_width =
            crate::text::cell_width(display_title, crate::text::TabPolicy::SingleCell);

        // Calculate position based on alignment
        let left_padding = match self.title_alignment {
            TitleAlignment::Left => 1,
            TitleAlignment::Center => {
                if title_width < available_width {
                    (available_width - title_width) / 2
                } else {
                    1
                }
            }
            TitleAlignment::Right => available_width.saturating_sub(title_width + 1),
        };

        let mut x = start_x;

        // Draw line before title
        for _ in 0..left_padding {
            window.write_str_colored(self.y, x, &chars.horizontal.to_string(), color)?;
            x += 1;
        }

        // Draw title with space before
        if left_padding > 0 {
            x -= 1;
            window.write_str_colored(self.y, x, " ", color)?;
            x += 1;
        }

        // Draw title
        window.write_str_colored(self.y, x, display_title, color)?;
        x += title_width;

        // Draw space and line after
        if x < start_x + available_width {
            window.write_str_colored(self.y, x, " ", color)?;
            x += 1;
        }

        for _ in x..(start_x + available_width) {
            window.write_str_colored(self.y, x, &chars.horizontal.to_string(), color)?;
            x += 1;
        }

        Ok(())
    }

    /// Draws the bottom border
    fn draw_bottom_border(
        &self,
        window: &mut dyn Window,
        chars: BorderChars,
        color: ColorPair,
    ) -> Result<()> {
        // Need at least 1 row to draw the bottom border.
        if self.height == 0 {
            return Ok(());
        }

        let y = self.y + self.height - 1;
        let has_left = self.border.sides.contains(&BorderSide::Left);
        let has_right = self.border.sides.contains(&BorderSide::Right);

        let mut x = self.x;

        // Draw left corner
        if has_left {
            window.write_str_colored(y, x, &chars.bottom_left.to_string(), color)?;
            x += 1;
        }

        // Draw horizontal line
        let end_x = self.x + self.width - if has_right { 1 } else { 0 };
        while x < end_x {
            window.write_str_colored(y, x, &chars.horizontal.to_string(), color)?;
            x += 1;
        }

        // Draw right corner
        if has_right {
            window.write_str_colored(
                y,
                self.x + self.width - 1,
                &chars.bottom_right.to_string(),
                color,
            )?;
        }

        Ok(())
    }

    /// Draws the left border
    fn draw_left_border(
        &self,
        window: &mut dyn Window,
        chars: BorderChars,
        color: ColorPair,
    ) -> Result<()> {
        let start_y = self.y
            + if self.border.sides.contains(&BorderSide::Top) {
                1
            } else {
                0
            };
        let end_y = self.y + self.height
            - if self.border.sides.contains(&BorderSide::Bottom) {
                1
            } else {
                0
            };

        for y in start_y..end_y {
            window.write_str_colored(y, self.x, &chars.vertical.to_string(), color)?;
        }

        Ok(())
    }

    /// Draws the right border
    fn draw_right_border(
        &self,
        window: &mut dyn Window,
        chars: BorderChars,
        color: ColorPair,
    ) -> Result<()> {
        // Need at least 1 column to draw the right border.
        if self.width == 0 {
            return Ok(());
        }

        let x = self.x + self.width - 1;
        let start_y = self.y
            + if self.border.sides.contains(&BorderSide::Top) {
                1
            } else {
                0
            };
        let end_y = self.y + self.height
            - if self.border.sides.contains(&BorderSide::Bottom) {
                1
            } else {
                0
            };

        for y in start_y..end_y {
            window.write_str_colored(y, x, &chars.vertical.to_string(), color)?;
        }

        Ok(())
    }

    /// Draws background fill
    fn draw_background(&self, window: &mut dyn Window) -> Result<()> {
        if !self.should_fill {
            return Ok(());
        }

        if let Some(color) = self.background_color {
            let (content_x, content_y, content_width, content_height) = self.get_content_area();

            for y in content_y..(content_y + content_height) {
                for x in content_x..(content_x + content_width) {
                    window.write_str_colored(y, x, " ", color)?;
                }
            }
        }

        Ok(())
    }

    /// Draws all children
    fn draw_children(&self, window: &mut dyn Window) -> Result<()> {
        let (content_x, content_y, content_width, content_height) = self.get_content_area();

        let gap = self.resolve_gap(match self.layout_direction {
            LayoutDirection::Vertical => content_height,
            LayoutDirection::Horizontal => content_width,
        });

        // ---- Main-axis fill resolution (even split, no weights yet) ----
        //
        // Strategy:
        // 1) Compute the main-axis space consumed by non-Fill children (Auto/Fixed) plus gaps.
        // 2) Split the remaining main-axis space evenly among Fill children.
        // 3) Apply per-child min/max constraints after resolution (clamped to remaining space).
        //
        // Notes:
        // - "Main axis" is Y for vertical containers and X for horizontal containers.
        // - Cross-axis Fill is still handled per-child during drawing.
        let child_count = self.children.len();
        let gap_total = if child_count > 1 {
            gap.saturating_mul((child_count - 1) as u16)
        } else {
            0
        };

        let (available_main, fill_count, fixed_main_total) = match self.layout_direction {
            LayoutDirection::Vertical => {
                let available = content_height.saturating_sub(gap_total);
                let mut fill = 0usize;
                let mut fixed_total: u16 = 0;

                for c in self.children.iter() {
                    match c.height {
                        SizeSpec::Fill => fill += 1,
                        SizeSpec::Fixed(h) => fixed_total = fixed_total.saturating_add(h),
                        SizeSpec::Auto => {
                            fixed_total = fixed_total.saturating_add(c.intrinsic_height)
                        }
                    }
                }

                (available, fill, fixed_total)
            }
            LayoutDirection::Horizontal => {
                let available = content_width.saturating_sub(gap_total);
                let mut fill = 0usize;
                let mut fixed_total: u16 = 0;

                for c in self.children.iter() {
                    match c.width {
                        SizeSpec::Fill => fill += 1,
                        SizeSpec::Fixed(w) => fixed_total = fixed_total.saturating_add(w),
                        SizeSpec::Auto => {
                            fixed_total = fixed_total.saturating_add(c.intrinsic_width)
                        }
                    }
                }

                (available, fill, fixed_total)
            }
        };

        let remaining_main = available_main.saturating_sub(fixed_main_total);

        // Even split: distribute remainder one cell at a time to the first N children
        // so the total exactly matches remaining_main.
        let (fill_each, fill_remainder) = if fill_count > 0 {
            (
                remaining_main / (fill_count as u16),
                remaining_main % (fill_count as u16),
            )
        } else {
            (0, 0)
        };

        let mut current_x = content_x;
        let mut current_y = content_y;

        // Tracks how many Fill children we've assigned so far (to dole out the remainder).
        let mut fill_assigned: u16 = 0;

        for (idx, child) in self.children.iter().enumerate() {
            let (intrinsic_w, intrinsic_h) = (child.intrinsic_width, child.intrinsic_height);

            // Remaining space in content area
            let remaining_width = content_width.saturating_sub(current_x - content_x);
            let remaining_height = content_height.saturating_sub(current_y - content_y);

            // Resolve sizes, then apply per-child constraints.
            let (mut resolved_w, mut resolved_h) = match self.layout_direction {
                LayoutDirection::Vertical => {
                    // Cross-axis (width): Fill means occupy remaining width.
                    let w = match child.width {
                        SizeSpec::Fill => remaining_width,
                        SizeSpec::Fixed(w) => w,
                        SizeSpec::Auto => intrinsic_w,
                    };

                    // Main-axis (height): Fill participates in even split.
                    let h = match child.height {
                        SizeSpec::Fill => {
                            let extra = if fill_assigned < fill_remainder { 1 } else { 0 };
                            fill_assigned = fill_assigned.saturating_add(1);
                            fill_each.saturating_add(extra)
                        }
                        SizeSpec::Fixed(h) => h,
                        SizeSpec::Auto => intrinsic_h,
                    };

                    (w, h)
                }
                LayoutDirection::Horizontal => {
                    // Cross-axis (height): Fill means occupy remaining height.
                    let h = match child.height {
                        SizeSpec::Fill => remaining_height,
                        SizeSpec::Fixed(h) => h,
                        SizeSpec::Auto => intrinsic_h,
                    };

                    // Main-axis (width): Fill participates in even split.
                    let w = match child.width {
                        SizeSpec::Fill => {
                            let extra = if fill_assigned < fill_remainder { 1 } else { 0 };
                            fill_assigned = fill_assigned.saturating_add(1);
                            fill_each.saturating_add(extra)
                        }
                        SizeSpec::Fixed(w) => w,
                        SizeSpec::Auto => intrinsic_w,
                    };

                    (w, h)
                }
            };

            // Apply constraints (then clamp to remaining space so we never draw outside).
            resolved_w = child
                .constraints
                .width
                .clamp(resolved_w)
                .min(remaining_width);
            resolved_h = child
                .constraints
                .height
                .clamp(resolved_h)
                .min(remaining_height);

            let mut child_view = WindowView {
                window,
                x_offset: current_x,
                y_offset: current_y,
                scroll_x: 0,
                scroll_y: 0,
                width: resolved_w,
                height: resolved_h,
            };

            child.widget.draw(&mut child_view)?;

            match self.layout_direction {
                LayoutDirection::Vertical => {
                    current_y += resolved_h;
                    if idx < self.children.len() - 1 {
                        current_y += gap;
                    }
                }
                LayoutDirection::Horizontal => {
                    current_x += resolved_w;
                    if idx < self.children.len() - 1 {
                        current_x += gap;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Container {
    fn draw(&self, window: &mut dyn Window) -> Result<()> {
        self.draw_background(window)?;
        self.draw_borders(window)?;
        self.draw_children(window)?;
        Ok(())
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}
