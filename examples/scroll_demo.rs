//! Scroll Demo
//!
//! Demonstrates MinUI's unified scrolling architecture:
//! - `ScrollBox` provides a scrollable viewport backed by a shared `ScrollState`
//! - `ScrollBar` binds to the same `ScrollState` for thumb dragging + arrow buttons
//! - Phase 1 interaction routing via `InteractionCache` + `WidgetArea`
//! - Scrollbar auto-hide UX via `AutoHide`
//!
//! Controls:
//! - Mouse wheel: vertical scroll
//! - Horizontal wheel (if your terminal sends it): horizontal scroll
//! - Drag scrollbar thumbs
//! - Click scrollbar arrows
//! - Arrow keys: scroll (Up/Down/Left/Right)
//! - Press 'q' to quit
//!
//! Notes:
//! - `ScrollBox` is sized each frame based on terminal size.
//! - Content is built each frame (no cloning required).
//! - Scrollbars are auto-hidden unless:
//!   - you recently scrolled, or
//!   - the mouse is near the scrollbar area, or
//!   - you are actively dragging the thumb.

use minui::prelude::*;
use minui::ui::{AutoHide, InteractionCache};
use minui::widgets::controls::scrollbar::{ScrollBar, ScrollBarOptions, ScrollUnit};
use minui::widgets::scroll::{ScrollOrientation, ScrollSize, ScrollState};
use minui::widgets::{WidgetArea, WindowView};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const ID_SCROLLBOX: usize = 1;
const ID_VSCROLL: usize = 2;
const ID_HSCROLL: usize = 3;

struct ScrollDemoState {
    ui: InteractionCache,
    scroll: Rc<RefCell<ScrollState>>,
    vbar: ScrollBar,
    hbar: ScrollBar,

    // Auto-hide behavior for scrollbars.
    // NOTE: This is app-level policy (Phase 1), not owned by the widgets.
    autohide: AutoHide,
}

fn main() -> minui::Result<()> {
    // Shared scroll model (content size + viewport size + offsets).
    // Seed with non-zero sizes so scrollbar math is sane before first draw.
    let scroll = Rc::new(RefCell::new(ScrollState::new(
        ScrollSize::new(1, 1),
        ScrollSize::new(1, 1),
    )));

    // Initial scrollbars; sizes are refreshed every draw frame.
    let vbar = ScrollBar::new(
        1,
        1,
        Rc::clone(&scroll),
        ScrollBarOptions {
            orientation: ScrollOrientation::Vertical,
            show_arrows: true,
            scroll_step: Some(1),
            ..ScrollBarOptions::default()
        },
    );

    let hbar = ScrollBar::new(
        1,
        1,
        Rc::clone(&scroll),
        ScrollBarOptions {
            orientation: ScrollOrientation::Horizontal,
            show_arrows: true,
            scroll_step: Some(2),
            ..ScrollBarOptions::default()
        },
    );

    let initial = ScrollDemoState {
        ui: InteractionCache::new(),
        scroll,
        vbar,
        hbar,
        autohide: AutoHide::new(Duration::from_millis(900), 2),
    };

    let mut app = App::new(initial)?;

    app.run(
        // ============================
        // Update: route events
        // ============================
        |state, event| {
            // Observe mouse position for proximity-based scrollbar reveal.
            state.ui.observe_event(&event);

            // Quit
            if matches!(event, Event::Character('q')) {
                return false;
            }

            // Mouse wheel scrolling -> update shared ScrollState directly.
            // Also mark "activity" so scrollbars reveal for a short time.
            match event {
                Event::MouseScroll { delta } => {
                    state.autohide.mark_activity();

                    // Convention: delta > 0 scrolls up, delta < 0 scrolls down.
                    // We invert to translate "scroll down" into positive offset movement.
                    let dy: i16 = -(delta as i16);
                    state
                        .scroll
                        .borrow_mut()
                        .scroll_by(ScrollOrientation::Vertical, dy);
                }
                Event::MouseScrollHorizontal { delta } => {
                    state.autohide.mark_activity();

                    let dx: i16 = -(delta as i16);
                    state
                        .scroll
                        .borrow_mut()
                        .scroll_by(ScrollOrientation::Horizontal, dx);
                }
                _ => {}
            }

            // Keyboard scrolling fallback.
            match event {
                Event::KeyUp => state
                    .vbar
                    .scroll_by_fraction(-1.0 / 5.0, ScrollUnit::Viewport),
                Event::KeyDown => state
                    .vbar
                    .scroll_by_fraction(1.0 / 5.0, ScrollUnit::Viewport),
                Event::KeyLeft => state
                    .hbar
                    .scroll_by_fraction(-1.0 / 5.0, ScrollUnit::Viewport),
                Event::KeyRight => state
                    .hbar
                    .scroll_by_fraction(1.0 / 5.0, ScrollUnit::Viewport),
                _ => {}
            }

            // Route pointer events into scrollbars (click/drag/arrows).
            // IMPORTANT: only route if the scrollbar was registered this frame (i.e. visible).
            // Areas come from the most recent draw frame.
            if let Some(entry) = state.ui.get(ID_VSCROLL) {
                let _ = state.vbar.handle_event(&event, entry.area);
            }
            if let Some(entry) = state.ui.get(ID_HSCROLL) {
                let _ = state.hbar.handle_event(&event, entry.area);
            }

            true
        },
        // ============================
        // Draw: register areas + draw
        // ============================
        |state, window| {
            state.ui.begin_frame();

            let (term_w, term_h) = window.get_size();

            // Layout constants
            let margin: u16 = 1;
            let header_h: u16 = 1;

            // Scrollbar sizes (vbar on the right, hbar on bottom)
            let vbar_w: u16 = 1;
            let hbar_h: u16 = 1;

            // ScrollBox styling affects its *outer* size vs the *inner* content viewport.
            // Because the ScrollBox draws a border + padding, we must reserve extra space so
            // the right/bottom borders don't get overwritten by scrollbars.
            let box_border: u16 = 1; // single-line border thickness
            let box_padding: u16 = 1; // we set ContainerPadding::uniform(1) below

            // Compute a safe outer rect for the ScrollBox (what you pass to with_position_and_size)
            // leaving room for:
            // - header line
            // - scrollbars (drawn OUTSIDE the ScrollBox frame)
            //
            // IMPORTANT:
            // - The ScrollBox outer rect must be fully within the terminal bounds.
            // - The scrollbars are then placed at `outer_x + outer_w` (right) and `outer_y + outer_h` (bottom),
            //   so we must reserve 1 extra column/row for them as well.
            let outer_x = margin;
            let outer_y = margin + header_h;

            // Total insets that reduce the usable inner content area.
            let inner_inset_x = box_border + box_padding;
            let inner_inset_y = box_border + box_padding;

            // Available space inside terminal margins for the ScrollBox outer rect.
            // We reserve the scrollbar column/row OUTSIDE the ScrollBox.
            let max_outer_w = term_w.saturating_sub(margin * 2).saturating_sub(vbar_w);
            let max_outer_h = term_h
                .saturating_sub(margin * 2)
                .saturating_sub(header_h)
                .saturating_sub(hbar_h);

            // Clamp outer size to available space. Do NOT force a minimum that can exceed terminal bounds.
            // If the terminal is too small, we'll end up with a very small box (and the scrollbars may be skipped).
            let outer_w = max_outer_w;
            let outer_h = max_outer_h;

            // Inner viewport (content area) starts after border+padding.
            // NOTE: these are currently unused in this example, but kept for clarity.
            let viewport_x = outer_x + inner_inset_x;
            let viewport_y = outer_y + inner_inset_y;
            let viewport_w = outer_w.saturating_sub(inner_inset_x * 2);
            let viewport_h = outer_h.saturating_sub(inner_inset_y * 2);

            let _ = (viewport_x, viewport_y, viewport_w, viewport_h);

            // Header / help line
            window.write_str(
                margin,
                margin,
                "Scroll demo: wheel/drag scrollbars • arrows/keys work • press 'q' to quit",
            )?;

            // Build content each frame (no cloning needed).
            // Use a row gap to exercise gap-aware content measurement in ScrollBox.
            let mut content = Container::vertical().with_row_gap(Gap::Pixels(1));

            // Lots of lines (taller than viewport).
            for i in 0..150u16 {
                content = content.add_child(
                    Label::new(format!(
                        "Line {:03} | The quick brown fox jumps over the lazy dog. {}",
                        i,
                        if i % 5 == 0 { "[drag the thumb]" } else { "" }
                    ))
                    .with_text_color(Color::White),
                );
            }

            // ScrollBox bound to shared state and sized to viewport.
            let scrollbox = ScrollBox::both()
                .with_state(Rc::clone(&state.scroll))
                // IMPORTANT: pass the OUTER rect (includes border/padding), not the inner viewport.
                .with_position_and_size(outer_x, outer_y, outer_w, outer_h)
                .with_border()
                .with_border_chars(BorderChars::single_line())
                .with_title("Scrollable content")
                .with_title_alignment(TitleAlignment::Left)
                .with_padding(minui::widgets::ContainerPadding::uniform(1))
                .with_row_gap(Gap::Pixels(1))
                .add_child(content);

            // Register scrollbox area (not strictly required for this demo, but useful for future focus/hover work).
            state.ui.register_scrollable(
                ID_SCROLLBOX,
                // Register the OUTER area for hover/click routing (matches draw bounds).
                WidgetArea::new(outer_x, outer_y, outer_w, outer_h),
            );

            // Draw scrollbox (updates ScrollState sizes + applies offsets via WindowView).
            scrollbox.draw(window)?;

            // Vertical scrollbar area (right side)
            // Place it just outside the ScrollBox's outer border so it doesn't overwrite the frame.
            // This must still be inside the terminal bounds (x < term_w).
            let vbar_x = outer_x + outer_w;
            let vbar_y = outer_y;
            let vbar_h = outer_h;

            // If there isn't room for the scrollbar column (e.g. tiny terminals), just skip drawing it.
            if vbar_x >= term_w {
                return Ok(());
            }

            // Resize + sync existing scrollbar instead of recreating it every frame.
            // Recreating would reset drag state, making thumb dragging feel broken.
            state.vbar.set_size(1, vbar_h);
            state.vbar.set_show_arrows(true);
            state.vbar.set_scroll_step(Some(1));
            state.vbar.sync_from_state_and_resize_parts();

            let v_area = WidgetArea::new(vbar_x, vbar_y, 1, vbar_h);

            // Auto-hide policy: show if recently scrolled, mouse is near, or currently dragging.
            // Reveal policy:
            // - show when recently scrolled
            // - show when mouse is near the scrollbar area
            // - show when mouse is near the scrollbox RIGHT edge strip (even if the bar itself is hidden)
            // - show while dragging
            let mouse_pos = state.ui.last_mouse_pos();

            // Only consider proximity to a thin strip along the scrollbox's right edge.
            // This prevents the scrollbars from staying visible just because the cursor is somewhere
            // inside/near the scrollbox.
            let near_scrollbox_right_edge = if let Some((mx, my)) = mouse_pos {
                // A 1-column strip at the right edge of the scrollbox.
                let edge_x = outer_x + outer_w.saturating_sub(1);
                let edge_strip = WidgetArea::new(edge_x, outer_y, 1, outer_h);
                state.autohide.is_point_near_area(mx, my, edge_strip)
            } else {
                false
            };

            let show_vbar = state
                .autohide
                .should_show(v_area, mouse_pos, state.vbar.is_dragging())
                || near_scrollbox_right_edge;

            if show_vbar {
                state.ui.register_draggable(ID_VSCROLL, v_area);

                {
                    let mut view = WindowView {
                        window,
                        x_offset: vbar_x,
                        y_offset: vbar_y,
                        scroll_x: 0,
                        scroll_y: 0,
                        width: 1,
                        height: vbar_h,
                    };
                    state.vbar.draw(&mut view)?;
                }
            }

            // Horizontal scrollbar area (bottom)
            // Place it just outside the ScrollBox's outer border so it doesn't overwrite the frame.
            // This must still be inside the terminal bounds (y < term_h).
            let hbar_x = outer_x;
            let hbar_y = outer_y + outer_h;
            let hbar_w = outer_w;

            // If there isn't room for the scrollbar row (e.g. tiny terminals), just skip drawing it.
            if hbar_y >= term_h {
                return Ok(());
            }

            // Resize + sync existing scrollbar instead of recreating it every frame.
            // Recreating would reset drag state, making thumb dragging feel broken.
            state.hbar.set_size(hbar_w, 1);
            state.hbar.set_show_arrows(true);
            state.hbar.set_scroll_step(Some(2));
            state.hbar.sync_from_state_and_resize_parts();

            let h_area = WidgetArea::new(hbar_x, hbar_y, hbar_w, 1);

            // Auto-hide policy: show if recently scrolled, mouse is near, or currently dragging.
            // Reveal policy:
            // - show when recently scrolled
            // - show when mouse is near the scrollbar area
            // - show when mouse is near the scrollbox BOTTOM edge strip (even if the bar itself is hidden)
            // - show while dragging
            let mouse_pos = state.ui.last_mouse_pos();

            // Only consider proximity to a thin strip along the scrollbox's bottom edge.
            // This prevents the scrollbars from staying visible just because the cursor is somewhere
            // inside/near the scrollbox.
            let near_scrollbox_bottom_edge = if let Some((mx, my)) = mouse_pos {
                // A 1-row strip at the bottom edge of the scrollbox.
                let edge_y = outer_y + outer_h.saturating_sub(1);
                let edge_strip = WidgetArea::new(outer_x, edge_y, outer_w, 1);
                state.autohide.is_point_near_area(mx, my, edge_strip)
            } else {
                false
            };

            let show_hbar = state
                .autohide
                .should_show(h_area, mouse_pos, state.hbar.is_dragging())
                || near_scrollbox_bottom_edge;

            if show_hbar {
                state.ui.register_draggable(ID_HSCROLL, h_area);

                {
                    let mut view = WindowView {
                        window,
                        x_offset: hbar_x,
                        y_offset: hbar_y,
                        scroll_x: 0,
                        scroll_y: 0,
                        width: hbar_w,
                        height: 1,
                    };
                    state.hbar.draw(&mut view)?;
                }
            }

            Ok(())
        },
    )?;

    Ok(())
}
