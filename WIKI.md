# MinUI user guide

MinUI provides a terminal window, an app loop, drawing widgets, and input-routing helpers. Your app owns its state and decides how events change it.

This guide follows the source in this repository, including changes that may not yet be in a published release. For a released crate, use the guide from its matching Git tag. See [Cargo.toml](Cargo.toml) for the repository's version and feature flags.

## Contents

- [Start with an app](#start-with-an-app)
- [Imports and optional features](#imports-and-optional-features)
- [The app loop](#the-app-loop)
- [Keyboard and paste input](#keyboard-and-paste-input)
- [Mouse input](#mouse-input)
- [Drawing and coordinates](#drawing-and-coordinates)
- [Layout and text widgets](#layout-and-text-widgets)
- [Interaction and routing](#interaction-and-routing)
- [Scrolling and controls](#scrolling-and-controls)
- [Text input](#text-input)
- [Other widgets](#other-widgets)
- [Unicode text utilities](#unicode-text-utilities)
- [Terminal capabilities](#terminal-capabilities)
- [Working examples and API reference](#working-examples-and-api-reference)

## Start with an app

After adding MinUI to your project as described in the [README](README.md#getting-started), run this program in a terminal. Press Space to increment the counter or Q to quit.

```rust
use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(0_u64)?;

    app.run(
        |count, event| {
            let event = event.as_legacy_key_event().unwrap_or(event);
            match event {
                Event::Character('q') => false,
                Event::Character(' ') => {
                    *count = count.saturating_add(1);
                    true
                }
                _ => true,
            }
        },
        |count, window| {
            Label::new(format!("Count: {count} | Space: increment | q: quit"))
                .draw(window)?;
            window.end_frame()
        },
    )
}
```

`App::new(state)` sets up a `TerminalWindow` and disables automatic flushing. The update closure receives mutable app state and an `Event`. The draw closure receives the same state and a `&mut dyn Window`. Finish each draw with `end_frame()` or `flush()`.

The window restores terminal modes when dropped. Creating a window requires a terminal, so app examples should be run interactively rather than inside an ordinary unit test.

## Imports and optional features

`use minui::prelude::*;` imports the common app, rendering, input, widget, and routing types. Smaller import bundles are available:

| Bundle | Includes |
| --- | --- |
| `minui::prelude::app::*` | `App`, `UpdateAction`, `FrameProfile`, events, window types, and results |
| `minui::prelude::render::*` | Drawing, colours, terminal capabilities, and text-width helpers |
| `minui::prelude::input::*` | Keyboard and mouse events, handlers, keybinds, and scrolling helpers |
| `minui::prelude::widgets::*` | Widgets, `WindowView`, `TextInput`, and `TextInputState` |
| `minui::prelude::interaction::*` | Hit testing, focus, capture, and routing |
| `minui::prelude::all::*` | The same broad bundle as `minui::prelude::*` |

Some useful types have explicit module paths:

- `minui::window::CursorSpec`
- `minui::widgets::ContainerPadding` and `ContainerContentAlignment`
- `minui::widgets::WindowView`, `TextInput`, and `TextInputState`

No optional features are enabled by default. The `figlet` feature adds `minui::widgets::FigletText`. The `clipboard` feature adds `minui::input::Clipboard`, whose `copy` and `paste` methods access the system clipboard. Enabling clipboard support does not automatically connect it to text-input widgets. Bracketed paste events work without that feature.

The [crate exports](src/lib.rs) and [widget exports](src/widgets/mod.rs) define the available import paths. The experimental [game module](src/game/mod.rs) is separate from the app and widget workflow covered here.

## The app loop

### Event-driven updates

`App::run(update, draw)` draws an initial frame, then blocks while waiting for input. It does not redraw continuously when idle.

The loop processes a bounded batch of terminal input before drawing. It combines adjacent mouse moves and resizes, and combines wheel deltas only when their axis and coordinates match. `Event::Unknown` is discarded. Resize batches update the window's dimensions and request a redraw.

With `run`, returning `true` from update requests a redraw; returning `false` exits. Multiple redraw requests in one batch result in one draw. Before drawing, the runner clears the drawing buffer, so draw the full visible UI each time. The terminal backend sends only the changed cells.

### Choosing when to redraw

Use `run_with_redraw` when some events leave the display unchanged:

```rust
use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut app = App::new(0_u64)?;

    app.run_with_redraw(
        |count, event| {
            let event = event.as_legacy_key_event().unwrap_or(event);
            match event {
                Event::Character('q') => UpdateAction::Exit,
                Event::Character(' ') => {
                    *count = count.saturating_add(1);
                    UpdateAction::Redraw
                }
                _ => UpdateAction::SkipRedraw,
            }
        },
        |count, window| {
            Label::new(format!("Count: {count}")).draw(window)?;
            window.end_frame()
        },
    )
}
```

A later `SkipRedraw` does not cancel an earlier `Redraw` in the same batch. The initial frame and resize batches draw even if update returns `SkipRedraw`.

### Timed updates and profiling

`with_frame_rate(Duration)` enables `Event::Frame` at the requested interval. The duration must be non-zero. The loop waits until input arrives or the next tick is due, so input remains responsive between ticks. Advance animations from `Frame`; with `run_with_redraw`, return `Redraw` when the animation changes.

`with_frame_budget(Duration)` sets a profiling threshold. It does not pace the app or add a sleep. Use `with_frame_rate` for timed updates.

`set_frame_profile_hook` receives a `FrameProfile` for rendered frames, including the initial frame. It reports input polling, update, draw, and frame durations, event count, budget, and whether the budget was exceeded. Input polling can include idle waiting; it is not a CPU-use measurement. Remove the hook with `clear_frame_profile_hook`.

See [app.rs](src/app.rs) and the [app runner example](examples/app_runner.rs).

## Keyboard and paste input

The app's keyboard handler emits `Event::KeyWithModifiers(KeyWithModifiers)` for supported keys unless a registered keybind matches first. `KeyWithModifiers` contains a `KeyKind` and `KeyModifiers` with `shift`, `ctrl`, `alt`, and `super_key` flags.

`KeyKind` covers characters, arrows, editing keys, Tab, Enter, Escape, Caps Lock, and function keys. For simple apps, `event.as_legacy_key_event().unwrap_or(event)` converts modifier-aware keys to variants such as `Character`, `KeyLeft`, and `Enter`. This drops modifiers; inspect the original event when implementing shortcuts or Shift-selection.

The app does not emit both a legacy event and a modifier-aware event for each key press. Standalone keyboard polling methods return legacy events, while `TerminalWindow` and `CombinedInputHandler` use the modifier-aware path.

Configure shortcuts before starting the app:

```rust
use minui::{App, KeybindAction};

fn configure_shortcuts(app: &mut App<()>) -> minui::Result<()> {
    let keyboard = app.window_mut().keyboard_mut();
    keyboard.add_keybind("ctrl-q", KeybindAction::Quit)?;
    keyboard.add_keybind("ctrl-s", KeybindAction::Save)?;
    Ok(())
}
```

A matching shortcut arrives as `Event::Keybind(action)`. The app decides what each action does. `App::new` does not install the optional common keybind set automatically.

Bracketed paste arrives as `Event::Paste(String)`, allowing an editor to insert it as one operation. Text characters already reflect the reported Shift and Caps Lock state. Do not toggle letter case locally in response to `Event::CapsLock`; terminals do not consistently report that key or its lock state.

For a custom input loop, use `TerminalWindow::poll_input`, `poll_input_timeout`, or `wait_for_input` to receive keyboard, mouse, paste, and resize events. `TerminalWindow::get_input` is a keyboard-only legacy convenience. Avoid polling separate keyboard and mouse handlers against the same terminal queue, because either can consume the other's events.

See [events](src/event.rs) and the [keyboard handler](src/input/keyboard.rs).

## Mouse input

All pointer coordinates are zero-based terminal cells: `x` is the column and `y` is the row.

| Event | Fields |
| --- | --- |
| `MouseMove` | `x`, `y` |
| `MouseClick`, `MouseDrag`, `MouseRelease` | `x`, `y`, `button` |
| `MouseScroll` | `x`, `y`, `delta: i8` |
| `MouseScrollHorizontal` | `x`, `y`, `delta: i8` |

### Wheel deltas and filtering

By default, vertical deltas are positive for down and negative for up. Horizontal deltas are positive for left and negative for right. `set_invert_scroll_vertical(true)` and `set_invert_scroll_horizontal(true)` reverse those signs.

Wheel events carry coordinates even when movement tracking is disabled. `UiScene::route_wheel_event` therefore works without a preceding mouse move.

Axis filtering is enabled by default. After scrolling on one axis, the first event on the other axis becomes `Event::Unknown`. A second consecutive event on that new axis is accepted with its own delta and coordinates. The discarded delta is never replayed. Returning to the active axis resets the pending switch.

If the app already filters wheel input, use `set_scroll_axis_filtering(false)` so both axes pass through immediately. Changing this setting clears the active axis and pending switch. Setting the same value again preserves filter state. Coordinates and delta inversion still apply. The setting is readable through `is_scroll_axis_filtering_enabled()`.

When migrating code that predates wheel coordinates, use `{ delta, .. }` in matches that only need the delta. Code constructing either wheel variant must supply `x` and `y`.

### Movement tracking and terminal capture

```rust
use minui::TerminalWindow;

fn configure_mouse(window: &mut TerminalWindow) -> minui::Result<()> {
    window.mouse_mut().set_movement_tracking(false);
    window.mouse_mut().set_scroll_axis_filtering(false);
    window.set_mouse_capture(true)?;
    Ok(())
}
```

Movement tracking starts enabled. Disabling it suppresses hover movement while keeping clicks, drags, releases, and wheel input. While capture is enabled, `TerminalWindow` applies this setting before its next input read. On Unix it selects terminal click-and-drag reporting. On Windows, movement filtering happens after console input is received.

Terminal mouse capture also starts enabled. `window.set_mouse_capture(false)?` disables reporting immediately. Input reads and movement-setting changes leave capture disabled. `set_mouse_capture(true)?` restores it with the current movement setting. `is_mouse_capture_enabled()` reports the window's capture state.

Use these methods instead of issuing Crossterm capture commands directly, which would bypass MinUI's cached state. Terminal reporting is separate from `UiScene` capture, which chooses the widget that receives a drag.

Standalone input handlers can return `Event::Unknown` for filtered input. The app runner discards it; custom loops should ignore it too.

See the [mouse handler](src/input/mouse.rs) and [input demo](examples/input_demo.rs).

## Drawing and coordinates

`Window` is the drawing interface. `TerminalWindow` implements it for the terminal; `WindowView` implements it for a clipped region.

The drawing methods put the row first: `write_str(y, x, text)` and `write_str_colored(y, x, text, colors)`. By contrast, mouse events, `WidgetArea::new(x, y, width, height)`, and cursor positions use `x` then `y`. Dimensions and offsets are terminal cells, not pixels or byte counts.

Useful methods include:

- `get_size()` returns width and height.
- `write_spans_colored(y, x, spans)` writes adjacent `ColoredSpan` values on one row.
- `clear_screen()` clears the drawing buffer.
- `clear_area(y1, x1, y2, x2)` clears an inclusive rectangle.
- `flush()` sends buffered changes; `end_frame()` also supports the deferred-cursor workflow.

`App` clears the drawing buffer before each draw. With a standalone `TerminalWindow`, call `set_auto_flush(false)` if you want to batch drawing yourself. `TerminalWindow::clear()` immediately clears the terminal and resets the buffer.

### The cursor

Widgets can request a cursor using `window.request_cursor(CursorSpec { x, y, visible })`, with `CursorSpec` imported from `minui::window`. The last request wins and is applied when the frame is flushed.

`clear_cursor_request()` discards a pending request. It does not hide an already visible cursor. To draw a frame with no cursor, explicitly request `visible: false`; a focused text input can then override that request while drawing.

### Drawing inside a region

Construct a `WindowView` with a struct literal. It has no `new` constructor:

```rust
use minui::prelude::*;

fn draw_region(window: &mut dyn Window, area: WidgetArea, scroll_y: u16) -> minui::Result<()> {
    let mut view = WindowView {
        window,
        x_offset: area.x,
        y_offset: area.y,
        scroll_x: 0,
        scroll_y,
        width: area.width,
        height: area.height,
    };
    TextBlock::new(area.width, 3, "First line\nSecond line\nThird line")
        .draw(&mut view)
}
```

The view subtracts scroll offsets from content coordinates, adds its position in the parent, and clips to its rectangle. A string starting left of the visible area can still draw its visible suffix. Rows above the viewport are skipped. Clipping preserves complete graphemes.

Register interactive regions in absolute terminal coordinates, even when their content is drawn through a view. At the top level, use the terminal's dimensions to keep each view inside its parent. Flush the outer window once after drawing all regions.

See [window.rs](src/window.rs) and [WindowView](src/widgets/common.rs).

## Layout and text widgets

Most drawing widgets implement `Widget`, which provides `draw`, `get_size`, and `get_position`, plus geometry helpers. `TextInput` has a separate drawing API that borrows its state, described below.

### Containers

`Container` arranges children vertically or horizontally and draws borders, titles, padding, and backgrounds. Give a top-level container an explicit rectangle or use `Container::fullscreen()`.

```rust
use minui::prelude::*;

fn draw_panel(window: &mut dyn Window) -> minui::Result<()> {
    let (width, height) = window.get_size();
    Container::vertical()
        .with_position_and_size(0, 0, width, height)
        .with_border()
        .with_title("Messages")
        .with_padding(ContainerPadding::uniform(1))
        .with_row_gap(Gap::Pixels(1))
        .add_child(Label::new("Inbox").with_text_color(Color::Cyan))
        .add_child(Text::new("No new messages"))
        .draw(window)
}
```

`ContainerPadding::uniform`, `symmetric`, and `custom` configure spacing. `Gap::Pixels` means terminal cells despite its name; `Gap::Percent` expresses a percentage. `add_child` uses the child's measured size. `add_child_fill` and `add_child_fill_weight` distribute remaining space. Width and height constraints can be set on the last child.

Children passed to a container must be owned and `'static`. Keep persistent app state outside the temporary layout. Reuse controls that retain drag or animation state rather than recreating them during every draw.

See [container.rs](src/widgets/container.rs).

### Labels and text blocks

`Label` and `Text` are single-line widgets. Both accept text through `new`, support `with_color`, `with_text_color`, and `with_alignment`, and expose `set_text`, `text`, and `cell_width`. Their `get_length` method counts characters, not display cells.

`TextBlock::new(width, height, text)` renders multiple lines. It supports horizontal and vertical alignment, explicit line breaks, and three wrapping modes:

| Mode | Behaviour |
| --- | --- |
| `TextWrapMode::None` | Keep lines unwrapped and clip to the available width |
| `TextWrapMode::Wrap` | Wrap at grapheme boundaries using terminal cell widths |
| `TextWrapMode::WrapWords` | Prefer word boundaries; split long words at grapheme boundaries |

Use `with_word_wrap()` as a convenience for `WrapWords`. `auto_sized` and `auto_sized_with_word_wrap` calculate dimensions from content. A text block does not own scroll offsets; draw it through a `WindowView` or inside a `ScrollBox` for scrolling.

See [text widgets](src/widgets/text.rs).

## Interaction and routing

Drawing a widget does not register event handlers. The app registers interactive rectangles and forwards events to the relevant state or control.

### Per-frame registration

Store a `UiScene` in app state. In each draw, call `begin_frame()`, then register the current absolute rectangles. Register the same logical widget with the same id across frames. `IdAllocator` can allocate ids once when constructing app state.

`UiScene::begin_frame` clears regions, tab order, and owner mappings. It preserves focus, capture, and the last mouse position. Clear focus or capture explicitly if the corresponding widget disappears.

Registration methods are `register`, `register_focusable`, `register_scrollable`, `register_draggable`, and `register_entry`. Entries can carry several flags. Higher `z_index` wins hit testing; at the same index, the last registration wins.

`InteractionCache` provides the underlying registry, hit testing, cached pointer position, and focus. `UiScene` adds tab traversal, capture, owner grouping, and routing policies. Register focusable regions through `UiScene` when you want its tab traversal; registering directly through `cache_mut()` does not populate the scene's tab order.

### Focus and capture

`apply_policies(&event)` updates the cached pointer position, handles optional Tab or Shift+Tab navigation, and applies click-to-focus and capture rules. By default, a left click focuses a focusable hit and captures a draggable hit. A left release clears capture.

`focus`, `clear_focus`, `focus_next`, and `focus_prev` support app-controlled focus. `set_capture` and `release_capture` support app-controlled drags. The `set_focus_under_cursor_on_click`, `set_tab_navigation_enabled`, and capture-policy setters allow individual policies to be disabled.

For a captured release, resolve and deliver its target before applying the policy that releases capture. Otherwise, a release outside the widget can miss the control that began the drag. Only pass pointer events to the mouse-routing helpers; an active capture takes precedence over hit testing.

### Routing helpers

Routing returns an `Option<RouteTarget>`; it does not deliver the event:

| Method | Decision |
| --- | --- |
| `route_mouse_event` | Captured id, otherwise the hit id for a move, click, drag, or release |
| `route_mouse_event_to_owner` | The same target, upgraded to an owner group when mapped |
| `route_wheel_event` | Scrollable hit, then its registered scrollable owner, then focused scrollable id |
| `route_wheel_event_to_owner` | The chosen scrollable id, upgraded if that id has an owner mapping |
| `route_key_event` | Focused id for a keyboard or paste event |

Wheel routing uses the coordinates in the wheel event and updates the cached mouse position. Other pointer events also update the cache when passed to `observe_event` or `apply_policies`.

### Scrollable owners

Use `set_owner(child_id, owner_id)` when a clickable child should forward wheel input to a scrollable region. This example can run without a terminal:

```rust
use minui::prelude::*;

fn main() {
    const PANEL: InteractionId = 1;
    const BUTTON: InteractionId = 2;
    let mut scene = UiScene::new();
    scene.begin_frame();
    scene.register_scrollable(PANEL, WidgetArea::new(0, 0, 40, 12));
    scene.register_focusable(BUTTON, WidgetArea::new(2, 2, 10, 1));
    scene.set_owner(BUTTON, PANEL);

    let wheel = Event::MouseScroll { x: 3, y: 2, delta: 1 };
    assert_eq!(scene.route_wheel_event(&wheel), Some(RouteTarget::Id(PANEL)));
}
```

The owner must be registered as scrollable in the current frame. An owner group id alone does not make it a wheel target. This fallback follows the hit child's direct mapping, without searching arbitrary regions behind it or walking an ancestor tree.

For composite controls, `set_owner_for_ids(owner, ids)` maps several parts to one controller. `RouteTarget::Owner` lets the app dispatch to that controller. Owner mappings are frame-scoped, so restore them after each `begin_frame()`.

`AutoHide` provides a separate visibility policy for scrollbars or similar controls. Call `mark_activity` when scrolling is handled, then use `should_show(area, mouse_position, is_dragging)`. It combines recent activity, pointer proximity, and drag state; it does not draw anything.

See [interaction.rs](src/ui/interaction.rs), [scene.rs](src/ui/scene.rs), and the [scroll demo](examples/scroll_demo.rs).

## Scrolling and controls

### Shared scroll state

`ScrollState::new(content_size, viewport_size)` holds content dimensions, viewport dimensions, and a clamped `ScrollOffset`. Sizes and offsets are measured in cells. `set_content_size`, `set_viewport_size`, `set_offset`, `scroll_by`, and `scroll_to` update that state within its valid range.

`ScrollState::scroll_by` uses positive values for down and right. With the default mouse-handler signs, pass vertical wheel deltas directly and negate horizontal wheel deltas. Widen an `i8` delta to `i16` before negating or scaling it.

`ScrollBox` draws a static frame and clipped scrolling content. It updates shared content and viewport dimensions during drawing. Keep the `Rc<RefCell<ScrollState>>` in app state so offsets survive rebuilt layouts.

```rust
use minui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn draw_scrollbox(
    window: &mut dyn Window,
    scene: &mut UiScene,
    scroll: Rc<RefCell<ScrollState>>,
    outer: WidgetArea,
    id: InteractionId,
) -> minui::Result<()> {
    let content = TextBlock::new(30, 3, "One\nTwo\nThree");
    let scrollbox = ScrollBox::vertical()
        .with_state(scroll)
        .with_position_and_size(outer.x, outer.y, outer.width, outer.height)
        .with_border()
        .with_title("History")
        .add_child(content);
    scrollbox.register_with_id(scene.cache_mut(), outer, id);
    scrollbox.draw(window)
}
```

Pass the outer rectangle, including borders and padding, to `register_with_id`. It registers the inner viewport as scrollable. `register_with_ids` registers a focusable outer frame and a scrollable viewport under separate ids. Neither helper begins a new frame.

`WindowView` is useful when the app already owns the offsets and only needs translation and clipping. `Viewport` additionally stores scroll position and dimensions, can attach to shared `ScrollState`, and can draw scroll indicators. For wheel handling with explicit direction conventions, update shared `ScrollState` as described above or call the viewport's `scroll_vertical` and `scroll_horizontal` methods with content-offset deltas.

### Sliders and scrollbars

`Slider::horizontal(width)` and `Slider::vertical(height)` create draggable numeric controls. Set the range with `with_range`, inspect `value`, and route pointer events to `handle_event(&event, absolute_area)`. Keep the slider instance in app state so drag state survives redraws.

`ScrollBar::vertical(height, shared_state)` and `ScrollBar::horizontal(width, shared_state)` connect a slider and optional arrow buttons to `ScrollState`. Before drawing, call `sync_from_state_and_resize_parts()` so the thumb reflects current content sizes and offsets. `register_with_id` and `register_with_ids` perform this synchronization internally.

Register scrollbar parts, map them to a controller owner, and forward routed events to `handle_event`. Use `set_size` when the terminal changes size. Keyboard scrolling can call `scroll_by_cells` or `scroll_by_fraction` with a `ScrollUnit`.

`ScrollBox` also provides sticky-edge and auto-scroll options. If using its auto-scroll methods, retain the `ScrollBox` instance and drive `update_auto_scroll` from timed updates; rebuilding the widget loses its internal animation state.

See [ScrollState](src/widgets/scroll/state.rs), [ScrollBox](src/widgets/scrollbox.rs), [Slider](src/widgets/controls/slider.rs), and [ScrollBar](src/widgets/controls/scrollbar.rs). The [scroll demo](examples/scroll_demo.rs) combines shared state, scrollbars, owner routing, and auto-hide.

## Text input

`TextInput` draws a single-line editor; `TextInputState` owns text, cursor, selection, horizontal view offset, and focus. Construct the widget with `TextInput::new().with_width(width)`. Draw it with `draw(window, &mut state)` and send keyboard or paste events to `state.handle_event(event)`.

The state only accepts input while focused. This complete example starts focused and uses Ctrl+Q to quit without reserving a printable character:

```rust
use minui::prelude::*;

fn main() -> minui::Result<()> {
    let mut input = TextInputState::new();
    input.set_focused(true);
    let mut app = App::new(input)?;
    app.window_mut().mouse_mut().set_movement_tracking(false);
    app.window_mut().keyboard_mut().add_keybind("ctrl-q", KeybindAction::Quit)?;

    app.run(
        |state, event| {
            if event == Event::Keybind(KeybindAction::Quit) {
                return false;
            }
            state.handle_event(event);
            true
        },
        |state, window| {
            let width = window.get_size().0.min(40);
            TextInput::new()
                .with_width(width)
                .with_placeholder("Type here; Ctrl+Q quits")
                .draw(window, state)?;
            window.end_frame()
        },
    )
}
```

State methods include `text`, `set_text`, `clear`, `cursor`, `selection`, `select_all`, `insert_char`, `insert_str`, `backspace`, and `delete_forward`. Cursor and selection indices count characters, not UTF-8 bytes. Editing moves across grapheme boundaries.

For mouse editing, route clicks to `click_set_cursor(x)` and drags to `drag_select_to(x)` using the appropriate terminal column. Draw first so the state's cached geometry is current. `draw_with_id` registers the input with an `InteractionCache` and draws it, but does not synchronize `UiScene` focus or tab order automatically. When using a scene, register focusable regions through the scene and update each input state's focus from the scene's focused id.

`copy_selection` and `cut_selection` return selected text for the app to send to a clipboard provider. Paste text can be inserted with `insert_str`; bracketed paste is handled by `handle_event`.

See [input.rs](src/widgets/input.rs) and the [text-input demo](examples/text_input_demo.rs) for multiple fields and selection handling.

## Other widgets

| Widget | Construction and updates | Source |
| --- | --- | --- |
| `Table` | `Table::new(x, y, width, height)`, then `with_columns` and `with_rows`; `set_scroll` changes offsets | [table.rs](src/widgets/table.rs) |
| `StatusBar` | `StatusBar::new()` with `with_left`, `with_center`, and `with_right`; `with_position` selects top or bottom | [statusbar.rs](src/widgets/statusbar.rs) |
| `Spinner` | `Spinner::new()` or `Spinner::with_frames(&[&'static str])`; `advance()` changes the frame | [spinner.rs](src/widgets/spinner.rs) |
| `Tooltip` | `Tooltip::new(text)` with `with_delay`, `at`, or `draw_at`; the app decides when to draw it | [tooltip.rs](src/widgets/tooltip.rs) |
| `FigletText` | `minui::widgets::FigletText::standard(text)?`, with the `figlet` feature | [figlet.rs](src/widgets/figlet.rs) |

A table and status bar can be drawn together without a container:

```rust
use minui::prelude::*;

fn draw_summary(window: &mut dyn Window) -> minui::Result<()> {
    let (width, height) = window.get_size();
    Table::new(0, 0, width, height.saturating_sub(1))
        .with_columns(vec![TableColumn::new("Task").with_width(24)])
        .with_rows(vec![vec![String::from("Review changes")]])
        .draw(window)?;
    StatusBar::new().with_left("Ready").with_right("Ctrl+Q: quit").draw(window)
}
```

Keep a `Spinner` in app state and call `advance` on `Event::Frame`. For delayed tooltips, update a `HoverTracker` with `start_hover` and `end_hover`, then query `should_show_tooltip(tooltip.delay())`. Enable timed updates if the tooltip should appear after a delay while the pointer is stationary.

## Unicode text utilities

Use terminal cell widths for layout. Byte length, character count, grapheme count, and displayed width are different quantities, especially for combining marks, wide characters, and emoji.

`cell_width` measures text in terminal cells. `clip_to_cells` returns a clipped `String`; `clip_to_cells_cow` can borrow unchanged text; `clip_to_cells_into` writes into a reusable buffer. `fit_to_cells` pads or truncates to a target width. These functions take a `TabPolicy` to make tab handling explicit.

`wrap_to_cells` produces displayed lines. `wrap_ranges_to_cells` returns source byte ranges, useful when carrying syntax colours through wrapping:

```rust
use minui::text::{TabPolicy, TextWrapMode, wrap_ranges_to_cells, wrap_to_cells};

fn main() {
    let source = "Hello 🧑‍💻 world";
    let lines = wrap_to_cells(source, 8, TextWrapMode::WrapWords, TabPolicy::SingleCell);
    assert_eq!(lines, ["Hello 🧑‍💻", "world"]);

    let ranges = wrap_ranges_to_cells(source, 8, TextWrapMode::WrapWords, TabPolicy::SingleCell);
    assert_eq!(&source[ranges[1].clone()], "world");
}
```

Both respect explicit line breaks and whole graphemes. Word wrapping trims line-edge whitespace while retaining spacing between words that fit together. Zero width produces no lines. Range results preserve source text, including tabs and oversized graphemes; the string helper expands tabs, removes controls, and clips graphemes that cannot fit.

The char-index and grapheme-index helpers in [text/mod.rs](src/text/mod.rs) convert between byte positions, logical indices, and cell columns. Use one indexing convention consistently within an editor. See also [text/wrap.rs](src/text/wrap.rs).

## Terminal capabilities

`TerminalWindow` detects `TerminalCapabilities` and downgrades colours when needed. `ColorSupport` distinguishes `Ansi16`, `Ansi256`, and `Truecolor`. Inspect the result with `window.capabilities()` or override it with `set_capabilities`.

The capability flags for mouse, bracketed paste, and cursor-style support are informational. Setting `capabilities.mouse` does not enable or disable reporting; use `set_mouse_capture` for that. Detection is best-effort, and terminal settings can affect input and display behaviour.

See [capabilities.rs](src/term/capabilities.rs).

## Working examples and API reference

Run examples from the repository with `cargo run --example NAME`:

| Example | Demonstrates |
| --- | --- |
| [basic_usage](examples/basic_usage.rs) | App setup and basic drawing |
| [app_runner](examples/app_runner.rs) | App-loop and timed-update patterns |
| [input_demo](examples/input_demo.rs) | Keyboard and mouse events |
| [scroll_demo](examples/scroll_demo.rs) | Scrollboxes, scrollbars, routing, and shared state |
| [text_input_demo](examples/text_input_demo.rs) | Focused fields and selection |
| [table_demo](examples/table_demo.rs) | Tables and scrolling |
| [syntax_profile_demo](examples/syntax_profile_demo.rs) | Coloured spans and rendering measurements |

For method signatures matching the checkout, generate the API reference locally:

```sh
cargo doc --no-deps --all-features --open
```

When building a custom widget, batch row writes, clip once, and flush once per frame. Keep persistent state outside temporary layouts, register regions each draw, and route input against the most recently drawn geometry.
