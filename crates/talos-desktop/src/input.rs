//! Presentation-local text input using GPUI's input-handler contract.
use crate::presentation::{Locale, Text};
use std::ops::Range;
use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, KeyBinding, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, Style, TextRun, UTF16Selection, UnderlineStyle, Window, actions, div, fill, point,
    prelude::*, px, relative, rgb, rgba, size,
};
use unicode_segmentation::UnicodeSegmentation;

actions!(
    desktop_input,
    [
        Left,
        Right,
        SelectLeft,
        SelectRight,
        Up,
        Down,
        SelectUp,
        SelectDown,
        Home,
        End,
        SelectAll,
        Backspace,
        Delete,
        Copy,
        Cut,
        Paste,
        Newline,
        CancelComposition
    ]
);

const INPUT_CONTEXT: &str = "DesktopMockInput";

#[derive(Clone, Debug, Default)]
struct TextModel {
    multiline: bool,
    text: String,
    anchor: usize,
    cursor: usize,
    upstream: bool,
    marked: Option<Range<usize>>,
    before_composition: Option<(String, usize, usize, bool)>,
}

impl TextModel {
    fn copy_to_clipboard(&mut self, cut: bool, write: impl FnOnce(String)) {
        let selection = self.selection();
        if selection.is_empty() {
            return;
        }
        let text = self.text[selection].to_owned();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| write(text))) {
            Ok(()) if cut => self.replace(None, "", false, None),
            Ok(()) => {}
            Err(_) => eprintln!("Desktop clipboard write failed; input retained"),
        }
    }

    fn paste_from_clipboard(&mut self, read: impl FnOnce() -> Option<String>) {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(read)) {
            Ok(Some(text)) => self.replace(None, &text, false, None),
            Ok(None) => {}
            Err(_) => eprintln!("Desktop clipboard read failed; input retained"),
        }
    }

    fn line_range(&self, position: usize) -> Range<usize> {
        if !self.multiline {
            return 0..self.text.len();
        }
        let start = self.text[..position]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let end = self.text[position..]
            .find('\n')
            .map_or(self.text.len(), |index| position + index);
        start..end
    }

    fn vertical_destination(&self, down: bool) -> usize {
        let current = self.line_range(self.cursor);
        let column = self.text[current.start..self.cursor]
            .graphemes(true)
            .count();
        let target = if down {
            if current.end == self.text.len() {
                return self.cursor;
            }
            self.line_range(current.end + 1)
        } else {
            if current.start == 0 {
                return self.cursor;
            }
            self.line_range(current.start - 1)
        };
        self.text[target.clone()]
            .grapheme_indices(true)
            .nth(column)
            .map_or(target.end, |(byte, _)| target.start + byte)
    }

    fn normalized(&self, text: &str) -> String {
        if self.multiline {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.chars()
                .map(|ch| if matches!(ch, '\r' | '\n') { ' ' } else { ch })
                .collect()
        }
    }
    fn select_accessible(&mut self, snapshot: &str, anchor: usize, cursor: usize) -> bool {
        if self.text != snapshot {
            return false;
        }
        let positions: Vec<_> = self
            .text
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(self.text.len()))
            .collect();
        let (Some(&anchor), Some(&cursor)) = (positions.get(anchor), positions.get(cursor)) else {
            return false;
        };
        // AccessKit exposes scalar offsets; preserve them exactly, independently of
        // the keyboard's grapheme navigation policy (as with IME UTF-16 ranges).
        self.unmark();
        self.anchor = anchor;
        self.cursor = cursor;
        self.upstream = false;
        true
    }

    fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }

    fn byte_from_utf16(text: &str, units: usize) -> usize {
        let mut count = 0;
        for (byte, ch) in text.char_indices() {
            if count + ch.len_utf16() > units {
                return byte;
            }
            count += ch.len_utf16();
        }
        text.len()
    }

    fn from_utf16(text: &str, range: Range<usize>) -> Range<usize> {
        let start = Self::byte_from_utf16(text, range.start.min(range.end));
        let end = Self::byte_from_utf16(text, range.start.max(range.end));
        start..end
    }

    fn to_utf16(&self, range: Range<usize>) -> Range<usize> {
        self.text[..range.start].encode_utf16().count()
            ..self.text[..range.end].encode_utf16().count()
    }

    fn previous(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .rev()
            .find_map(|(byte, _)| (byte < self.cursor).then_some(byte))
            .unwrap_or(0)
    }

    fn next(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .find_map(|(byte, _)| (byte > self.cursor).then_some(byte))
            .unwrap_or(self.text.len())
    }

    fn move_to(&mut self, byte: usize, selecting: bool) {
        let byte = self
            .text
            .grapheme_indices(true)
            .map(|(byte, _)| byte)
            .chain(std::iter::once(self.text.len()))
            .take_while(|candidate| *candidate <= byte)
            .last()
            .unwrap_or(0);
        self.cursor = byte;
        self.upstream = false;
        if !selecting {
            self.anchor = byte;
        }
        self.unmark();
    }

    fn replacement(&self, range: Option<Range<usize>>) -> Range<usize> {
        range
            .map(|range| Self::from_utf16(&self.text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection())
    }

    fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        marked: bool,
        selected_utf16: Option<Range<usize>>,
    ) {
        let range = self.replacement(range);
        // IME offsets refer to the incoming string before CRLF normalization.
        let selected_bytes = selected_utf16.map(|selection| {
            let local = Self::from_utf16(text, selection);
            self.normalized(&text[..local.start]).len()..self.normalized(&text[..local.end]).len()
        });
        let text = self.normalized(text);
        if marked && self.before_composition.is_none() {
            self.before_composition =
                Some((self.text.clone(), self.anchor, self.cursor, self.upstream));
        }
        self.text.replace_range(range.clone(), &text);
        let inserted = range.start..range.start + text.len();
        self.marked = (marked && !text.is_empty()).then_some(inserted.clone());
        let selected = selected_bytes
            .map(|local| range.start + local.start..range.start + local.end)
            .unwrap_or(inserted.end..inserted.end);
        self.anchor = selected.start;
        self.cursor = selected.end;
        self.upstream = false;
        if !marked {
            self.before_composition = None;
        }
    }

    fn unmark(&mut self) {
        self.marked = None;
        self.before_composition = None;
    }

    fn cancel_composition(&mut self) {
        if let Some((text, anchor, cursor, upstream)) = self.before_composition.take() {
            self.text = text;
            self.anchor = anchor;
            self.cursor = cursor;
            self.upstream = upstream;
        }
        self.marked = None;
    }

    fn delete(&mut self, backwards: bool) {
        if self.selection().is_empty() {
            self.anchor = if backwards {
                self.previous()
            } else {
                self.next()
            };
        }
        self.replace(None, "", false, None);
    }
}

pub(crate) struct TextInput {
    pub(crate) compact: bool,
    pub(crate) locale: Locale,
    pub(crate) label: Text,
    focus: FocusHandle,
    model: TextModel,
    layout: Option<Rc<InputLayout>>,
    bounds: Option<Bounds<Pixels>>,
    scroll_x: Pixels,
    scroll_y: Pixels,
    revealed: Option<(String, usize, bool)>,
    dragging: bool,
}

impl TextInput {
    pub(crate) fn multiline(mut self) -> Self {
        self.model.multiline = true;
        self
    }
    pub(crate) fn text(&self) -> &str {
        &self.model.text
    }
    #[cfg(feature = "visual-test")]
    pub(crate) fn scroll_offset(&self) -> Point<Pixels> {
        point(self.scroll_x, self.scroll_y)
    }
    pub(crate) fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let end = self.model.text.encode_utf16().count();
        self.model.replace(Some(0..end), text, false, None);
        self.dragging = false;
        self.scroll_x = px(0.);
        self.scroll_y = px(0.);
        self.revealed = None;
        cx.notify();
    }
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let context = Some(INPUT_CONTEXT);
        cx.bind_keys([
            KeyBinding::new("left", Left, context),
            KeyBinding::new("right", Right, context),
            KeyBinding::new("shift-left", SelectLeft, context),
            KeyBinding::new("shift-right", SelectRight, context),
            KeyBinding::new("up", Up, context),
            KeyBinding::new("down", Down, context),
            KeyBinding::new("shift-up", SelectUp, context),
            KeyBinding::new("shift-down", SelectDown, context),
            KeyBinding::new("home", Home, context),
            KeyBinding::new("end", End, context),
            KeyBinding::new("backspace", Backspace, context),
            KeyBinding::new("delete", Delete, context),
            KeyBinding::new("escape", CancelComposition, context),
            KeyBinding::new("enter", Newline, context),
        ]);
        #[cfg(target_os = "macos")]
        cx.bind_keys([
            KeyBinding::new("cmd-a", SelectAll, context),
            KeyBinding::new("cmd-c", Copy, context),
            KeyBinding::new("cmd-x", Cut, context),
            KeyBinding::new("cmd-v", Paste, context),
        ]);
        #[cfg(not(target_os = "macos"))]
        cx.bind_keys([
            KeyBinding::new("ctrl-a", SelectAll, context),
            KeyBinding::new("ctrl-c", Copy, context),
            KeyBinding::new("ctrl-x", Cut, context),
            KeyBinding::new("ctrl-v", Paste, context),
        ]);
        let focus = cx.focus_handle().tab_index(10).tab_stop(true);
        cx.on_focus(&focus, window, |input, _, cx| {
            input.revealed = None;
            cx.notify();
        })
        .detach();
        cx.on_blur(&focus, window, |input, _, cx| {
            input.model.unmark();
            input.dragging = false;
            cx.notify();
        })
        .detach();
        Self {
            compact: false,
            locale: Locale::English,
            label: Text::Draft,
            focus,
            model: TextModel::default(),
            layout: None,
            bounds: None,
            scroll_x: px(0.),
            scroll_y: px(0.),
            revealed: None,
            dragging: false,
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let position = if self.model.selection().is_empty() {
            self.model.previous()
        } else {
            self.model.selection().start
        };
        self.model.move_to(position, false);
        cx.notify();
    }
    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let position = if self.model.selection().is_empty() {
            self.model.next()
        } else {
            self.model.selection().end
        };
        self.model.move_to(position, false);
        cx.notify();
    }
    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.model.move_to(self.model.previous(), true);
        cx.notify();
    }
    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.model.move_to(self.model.next(), true);
        cx.notify();
    }
    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.model
            .move_to(self.model.line_range(self.model.cursor).start, false);
        cx.notify();
    }
    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.model
            .move_to(self.model.line_range(self.model.cursor).end, false);
        cx.notify();
    }
    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.model.move_to(0, false);
        self.model.move_to(self.model.text.len(), true);
        cx.notify();
    }
    fn vertical(&mut self, down: bool, selecting: bool, cx: &mut Context<Self>) {
        if !self.model.multiline {
            cx.propagate();
            return;
        }
        let destination = self
            .layout
            .as_ref()
            .filter(|layout| !layout.failed && layout.text == self.model.text)
            .map(|layout| {
                let position =
                    layout.position_with_affinity(self.model.cursor, self.model.upstream);
                layout.hit(point(
                    position.x,
                    position.y + layout.line_height * if down { 1. } else { -1. },
                ))
            })
            .unwrap_or_else(|| (self.model.vertical_destination(down), false));
        self.model.move_to(destination.0, selecting);
        self.model.upstream = destination.1;
        cx.notify();
    }
    fn newline(&mut self, _: &Newline, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.multiline || self.model.marked.is_some() {
            cx.propagate();
            return;
        }
        self.model.replace(None, "\n", false, None);
        cx.notify();
    }
    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.model.delete(true);
        cx.notify();
    }
    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.model.delete(false);
        cx.notify();
    }
    fn cancel(&mut self, _: &CancelComposition, _: &mut Window, cx: &mut Context<Self>) {
        self.model.cancel_composition();
        cx.notify();
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.model.copy_to_clipboard(false, |text| {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        });
    }
    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        self.model.copy_to_clipboard(true, |text| {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        });
        cx.notify();
    }
    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        self.model
            .paste_from_clipboard(|| cx.read_from_clipboard().and_then(|item| item.text()));
        cx.notify();
    }
    fn mouse_index(&self, position: Point<Pixels>) -> usize {
        self.mouse_hit(position).0
    }
    fn mouse_hit(&self, position: Point<Pixels>) -> (usize, bool) {
        match (&self.layout, self.bounds) {
            (Some(layout), Some(bounds)) if !layout.failed && layout.text == self.model.text => {
                layout.hit(point(
                    position.x - bounds.left() + self.scroll_x,
                    position.y - bounds.top() + self.scroll_y,
                ))
            }
            _ => (self.model.cursor, self.model.upstream),
        }
    }
    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        self.dragging = true;
        let (byte, upstream) = self.mouse_hit(event.position);
        self.model.move_to(byte, event.modifiers.shift);
        self.model.upstream = upstream;
        cx.notify();
    }
    fn mouse_up(&mut self, event: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragging {
            let (byte, upstream) = self.mouse_hit(event.position);
            self.model.move_to(byte, true);
            self.model.upstream = upstream;
            self.dragging = false;
            cx.notify();
        }
    }
    fn mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragging {
            let (byte, upstream) = self.mouse_hit(event.position);
            self.model.move_to(byte, true);
            self.model.upstream = upstream;
            cx.notify();
        }
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = TextModel::from_utf16(&self.model.text, range);
        *actual = Some(self.model.to_utf16(range.clone()));
        Some(self.model.text[range].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.model.to_utf16(self.model.selection()),
            reversed: self.model.cursor < self.model.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.model
            .marked
            .clone()
            .map(|range| self.model.to_utf16(range))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.model.unmark();
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.model.replace(range, text, false, None);
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selection: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.model.replace(range, text, true, selection);
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.layout.as_ref()?;
        if layout.failed || layout.text != self.model.text {
            return None;
        }
        let bounds = self.bounds?;
        let range = TextModel::from_utf16(&self.model.text, range);
        let start =
            layout.position_with_affinity(range.start, range.is_empty() && self.model.upstream);
        let end = layout.position_with_affinity(range.end, range.is_empty() && self.model.upstream);
        let left = (bounds.left() + start.x - self.scroll_x)
            .max(bounds.left())
            .min(bounds.right());
        let right = (bounds.left() + if start.y == end.y { end.x } else { start.x }
            - self.scroll_x)
            .max(left)
            .min(bounds.right());
        Some(Bounds::from_corners(
            point(
                left,
                (bounds.top() + start.y - self.scroll_y)
                    .max(bounds.top())
                    .min(bounds.bottom()),
            ),
            point(
                right,
                (bounds.top() + start.y + layout.line_height - self.scroll_y)
                    .max(bounds.top())
                    .min(bounds.bottom()),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        if self.layout.as_ref()?.failed {
            return None;
        }
        self.bounds?.localize(&point)?;
        let index = self.mouse_index(point);
        Some(self.model.to_utf16(index..index).start)
    }
}

type AccessibleRuns = Rc<RefCell<Vec<(Range<usize>, gpui::accesskit::Node)>>>;

struct InputLine {
    input: Entity<TextInput>,
    accessible_runs: AccessibleRuns,
}
struct LinePaint {
    layout: Rc<InputLayout>,
    cursor: PaintQuad,
    selection: Vec<PaintQuad>,
    origin: Point<Pixels>,
    scroll: Point<Pixels>,
}

struct InputRow {
    range: Range<usize>,
    // Include the hard newline in the accessible run, but not in glyph shaping.
    accessible_end: usize,
    line: ShapedLine,
}

struct InputLayout {
    text: String,
    rows: Vec<InputRow>,
    line_height: Pixels,
    failed: bool,
}

impl InputLayout {
    fn row_index(&self, byte: usize) -> usize {
        self.rows
            .iter()
            .rposition(|row| row.range.start <= byte)
            .unwrap_or(0)
    }

    #[cfg(test)]
    fn position(&self, byte: usize) -> Point<Pixels> {
        self.position_with_affinity(byte, false)
    }

    fn position_with_affinity(&self, byte: usize, upstream: bool) -> Point<Pixels> {
        let index = if upstream {
            self.rows
                .iter()
                .position(|row| row.range.end == byte)
                .unwrap_or_else(|| self.row_index(byte))
        } else {
            self.row_index(byte)
        };
        let row = &self.rows[index];
        point(
            row.line
                .x_for_index(byte.min(row.range.end) - row.range.start),
            self.line_height * index,
        )
    }

    #[cfg(test)]
    fn index(&self, position: Point<Pixels>) -> usize {
        self.hit(position).0
    }

    fn hit(&self, position: Point<Pixels>) -> (usize, bool) {
        let index = ((position.y.max(px(0.)) / self.line_height) as usize).min(self.rows.len() - 1);
        let row = &self.rows[index];
        let byte = row.range.start + row.line.closest_index_for_x(position.x);
        let upstream = byte == row.range.end
            && self
                .rows
                .get(index + 1)
                .is_some_and(|next| next.range.start == byte);
        (byte, upstream)
    }

    fn height(&self) -> Pixels {
        self.line_height * self.rows.len()
    }
}

fn runs_for_range(runs: &[TextRun], range: Range<usize>) -> Vec<TextRun> {
    let mut start = 0;
    runs.iter()
        .filter_map(|run| {
            let end = start + run.len;
            let len = end.min(range.end).saturating_sub(start.max(range.start));
            start = end;
            (len > 0).then(|| TextRun { len, ..run.clone() })
        })
        .collect()
}

fn accessible_position(
    text: &str,
    rows: &[(gpui::accesskit::NodeId, Range<usize>)],
    byte: usize,
    upstream: bool,
) -> Option<gpui::accesskit::TextPosition> {
    let previous = upstream
        .then(|| rows.iter().find(|(_, range)| range.end == byte))
        .flatten();
    let (id, range) =
        previous.or_else(|| rows.iter().rev().find(|(_, range)| range.start <= byte))?;
    Some(gpui::accesskit::TextPosition {
        node: *id,
        character_index: text[range.start..byte.min(range.end)].chars().count(),
    })
}

fn accessible_upstream(
    text: &str,
    rows: &[(gpui::accesskit::NodeId, Range<usize>)],
    position: &gpui::accesskit::TextPosition,
) -> bool {
    rows.iter().enumerate().any(|(index, (id, range))| {
        *id == position.node
            && position.character_index == text[range.clone()].chars().count()
            && !text[range.clone()].ends_with('\n')
            && rows
                .get(index + 1)
                .is_some_and(|(_, next)| next.start == range.end)
    })
}

fn accessible_scalar(
    text: &str,
    rows: &[(gpui::accesskit::NodeId, Range<usize>)],
    position: &gpui::accesskit::TextPosition,
) -> Option<usize> {
    let (_, range) = rows.iter().find(|(id, _)| *id == position.node)?;
    (position.character_index <= text[range.clone()].chars().count())
        .then(|| text[..range.start].chars().count() + position.character_index)
}

fn shape_input(
    text: &str,
    runs: &[TextRun],
    multiline: bool,
    width: Pixels,
    window: &Window,
) -> InputLayout {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        shape_input_unchecked(text, runs, multiline, width, window)
    })) {
        Ok(layout) => layout,
        Err(_) => {
            eprintln!("Desktop input shaping failed; text retained, geometry unavailable");
            fallback_layout(text, window.line_height())
        }
    }
}

fn fallback_layout(text: &str, line_height: Pixels) -> InputLayout {
    // No native calls on the failure path. Preserve editable text and AT value.
    InputLayout {
        text: text.to_owned(),
        rows: vec![InputRow {
            range: 0..text.len(),
            accessible_end: text.len(),
            line: ShapedLine::default(),
        }],
        line_height,
        failed: true,
    }
}

fn shape_input_unchecked(
    text: &str,
    runs: &[TextRun],
    multiline: bool,
    width: Pixels,
    window: &Window,
) -> InputLayout {
    let font_size = window.text_style().font_size.to_pixels(window.rem_size());
    let mut rows = Vec::new();
    let mut start = 0;
    // Shape each hard line separately. All consumers below use these same visual rows.
    for paragraph in text.split('\n') {
        let end = start + paragraph.len();
        let paragraph_runs = runs_for_range(runs, start..end);
        let mut starts = vec![0];
        if multiline {
            match window.text_system().shape_text(
                paragraph.to_owned().into(),
                font_size,
                &paragraph_runs,
                Some(width),
                None,
            ) {
                Ok(lines) => {
                    if let Some(line) = lines.first() {
                        starts.extend(line.wrap_boundaries().iter().map(|boundary| {
                            line.runs()[boundary.run_ix].glyphs[boundary.glyph_ix].index
                        }));
                    }
                }
                Err(error) => eprintln!("Desktop input wrapping unavailable: {error}"),
            }
        }
        starts.dedup();
        for (index, &local_start) in starts.iter().enumerate() {
            let local_end = starts.get(index + 1).copied().unwrap_or(paragraph.len());
            let range = start + local_start..start + local_end;
            let line = window.text_system().shape_line(
                text[range.clone()].to_owned().into(),
                font_size,
                &runs_for_range(runs, range.clone()),
                None,
            );
            let accessible_end = if local_end == paragraph.len() && end < text.len() {
                end + 1
            } else {
                range.end
            };
            rows.push(InputRow {
                range,
                accessible_end,
                line,
            });
        }
        start = end + 1;
    }
    InputLayout {
        text: text.to_owned(),
        rows,
        line_height: window.line_height(),
        failed: false,
    }
}

impl IntoElement for InputLine {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for InputLine {
    type RequestLayoutState = ();
    type PrepaintState = LinePaint;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height()
            * if self.input.read(cx).model.multiline {
                5.
            } else {
                1.
            })
        .into();
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> LinePaint {
        let input = self.input.read(cx);
        let style = window.text_style();
        let run = TextRun {
            len: input.model.text.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked) = &input.model.marked {
            vec![
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.len(),
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: input.model.text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect::<Vec<_>>()
        } else {
            vec![run]
        };
        let width = (bounds.size.width - px(2.)).max(px(1.));
        let layout = Rc::new(shape_input(
            &input.model.text,
            &runs,
            input.model.multiline,
            width,
            window,
        ));
        let caret = layout.position_with_affinity(input.model.cursor, input.model.upstream);
        let max_width = layout
            .rows
            .iter()
            .map(|row| row.line.width)
            .max()
            .unwrap_or(px(0.));
        let mut scroll = point(
            input.scroll_x.min((max_width - width).max(px(0.))),
            input
                .scroll_y
                .min((layout.height() - bounds.size.height).max(px(0.))),
        );
        let reveal = input
            .revealed
            .as_ref()
            .is_none_or(|(text, cursor, upstream)| {
                text != &input.model.text
                    || *cursor != input.model.cursor
                    || *upstream != input.model.upstream
            })
            || input.bounds.is_none_or(|old| old.size != bounds.size);
        if reveal && input.focus.is_focused(window) {
            scroll.x = scroll.x.min(caret.x).max(caret.x - width);
            scroll.y = scroll
                .y
                .min(caret.y)
                .max(caret.y + layout.line_height - bounds.size.height)
                .max(px(0.));
        }
        let origin = point(bounds.left() - scroll.x, bounds.top() - scroll.y);
        // The parent collects synthetic nodes after child prepaint. Share this
        // frame's shaped coordinates, never the previous paint's cached layout.
        let scale = window.scale_factor();
        let mut accessible_runs = Vec::new();
        let mut selection = Vec::new();
        let range = input.model.selection();
        for (index, row) in layout.rows.iter().enumerate() {
            let text = &layout.text[row.range.start..row.accessible_end];
            let offsets: Vec<f32> = text
                .char_indices()
                .map(|(byte, _)| f32::from(row.line.x_for_index(byte.min(row.range.len()))) * scale)
                .chain(std::iter::once(f32::from(row.line.width) * scale))
                .collect();
            let top = origin.y + layout.line_height * index;
            let mut accessible = gpui::accesskit::Node::new(gpui::Role::TextRun);
            accessible.set_value(text.to_owned());
            accessible.set_character_lengths(
                text.chars()
                    .map(|ch| ch.len_utf8() as u8)
                    .collect::<Vec<_>>(),
            );
            accessible.set_character_positions(offsets[..offsets.len() - 1].to_vec());
            accessible.set_character_widths(
                offsets
                    .windows(2)
                    .map(|pair| (pair[1] - pair[0]).max(0.))
                    .collect::<Vec<_>>(),
            );
            accessible.set_text_direction(gpui::accesskit::TextDirection::LeftToRight);
            accessible.set_bounds(gpui::accesskit::Rect {
                x0: f64::from(f32::from(origin.x) * scale),
                y0: f64::from(f32::from(top) * scale),
                x1: f64::from(f32::from(origin.x + row.line.width) * scale),
                y1: f64::from(f32::from(top + layout.line_height) * scale),
            });
            accessible_runs.push((row.range.start..row.accessible_end, accessible));
            let start = range.start.max(row.range.start);
            let end = range.end.min(row.accessible_end);
            if start < end {
                let left = row
                    .line
                    .x_for_index(start.min(row.range.end) - row.range.start);
                let right = row
                    .line
                    .x_for_index(end.min(row.range.end) - row.range.start)
                    + if end > row.range.end { px(5.) } else { px(0.) };
                selection.push(fill(
                    Bounds::from_corners(
                        point(origin.x + left, top),
                        point(origin.x + right, top + layout.line_height),
                    ),
                    rgba(0x2878cf40),
                ));
            }
        }
        *self.accessible_runs.borrow_mut() = accessible_runs;
        let cursor = fill(
            Bounds::new(
                point(origin.x + caret.x, origin.y + caret.y),
                size(px(2.), layout.line_height),
            ),
            rgb(0x2878cf),
        );
        LinePaint {
            layout,
            cursor,
            selection,
            origin,
            scroll,
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        state: &mut LinePaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus.clone();
        // Selection must follow the pointer outside the input hitbox until release.
        let input = self.input.downgrade();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == gpui::DispatchPhase::Bubble {
                let _ = input.update(cx, |input, cx| input.mouse_move(event, window, cx));
            }
        });
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for selection in state.selection.drain(..) {
                window.paint_quad(selection);
            }
            for (index, row) in state.layout.rows.iter().enumerate() {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    row.line.paint(
                        point(
                            state.origin.x,
                            state.origin.y + state.layout.line_height * index,
                        ),
                        state.layout.line_height,
                        gpui::TextAlign::Left,
                        None,
                        window,
                        cx,
                    )
                }));
                if !matches!(result, Ok(Ok(()))) || state.layout.failed {
                    eprintln!("Desktop mock input paint unavailable; text retained");
                    window.paint_quad(fill(
                        Bounds::new(bounds.origin, size(px(3.), bounds.size.height)),
                        rgb(0xb23a48),
                    ));
                }
            }
            if focus.is_focused(window) {
                window.paint_quad(state.cursor.clone());
            }
        });
        self.input.update(cx, |input, _| {
            input.layout = Some(state.layout.clone());
            input.bounds = Some(bounds);
            input.scroll_x = state.scroll.x;
            input.scroll_y = state.scroll.y;
            input.revealed = Some((
                input.model.text.clone(),
                input.model.cursor,
                input.model.upstream,
            ));
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let run_ids = Rc::new(RefCell::new(
            Vec::<(gpui::accesskit::NodeId, Range<usize>)>::new(),
        ));
        let accessible_runs = Rc::new(RefCell::new(
            Vec::<(Range<usize>, gpui::accesskit::Node)>::new(),
        ));
        let shaped_runs = accessible_runs.clone();
        let selection_runs = run_ids.clone();
        let text = self.model.text.clone();
        let selection_snapshot = text.clone();
        let anchor = self.model.anchor;
        let cursor = self.model.cursor;
        let upstream = self.model.upstream;
        let multiline = self.model.multiline;
        let value_target = cx.entity().downgrade();
        let replacement_target = cx.entity().downgrade();
        let selection_target = cx.entity().downgrade();
        div()
            .id("desktop-fixture-input")
            .role(if multiline {
                gpui::Role::MultilineTextInput
            } else {
                gpui::Role::TextInput
            })
            .accessibility_id("desktop-fixture-input")
            .aria_label(self.locale.text(self.label))
            .aria_value(self.model.text.clone())
            .a11y_synthetic_children(move |builder| {
                let mut mapping = run_ids.borrow_mut();
                mapping.clear();
                for (index, (range, run)) in shaped_runs.borrow_mut().drain(..).enumerate() {
                    let id = builder.synthetic_node_id(index as u64);
                    builder.push_child(id, run);
                    mapping.push((id, range));
                }
                let (Some(anchor), Some(focus)) = (
                    accessible_position(&text, &mapping, anchor, anchor == cursor && upstream),
                    accessible_position(&text, &mapping, cursor, upstream),
                ) else {
                    return;
                };
                builder
                    .parent_node()
                    .set_text_selection(gpui::accesskit::TextSelection { anchor, focus });
            })
            .on_a11y_action(gpui::AccessibleAction::SetValue, move |data, _, cx| {
                if let Some(gpui::accesskit::ActionData::Value(value)) = data {
                    let _ = value_target.update(cx, |input, cx| {
                        let end = input.model.text.encode_utf16().count();
                        input.model.replace(Some(0..end), value, false, None);
                        cx.notify();
                    });
                }
            })
            .on_a11y_action(
                gpui::AccessibleAction::SetTextSelection,
                move |data, _, cx| {
                    if let Some(gpui::accesskit::ActionData::SetTextSelection(selection)) = data {
                        let mapping = selection_runs.borrow();
                        let (Some(anchor), Some(cursor)) = (
                            accessible_scalar(&selection_snapshot, &mapping, &selection.anchor),
                            accessible_scalar(&selection_snapshot, &mapping, &selection.focus),
                        ) else {
                            return;
                        };
                        let _ = selection_target.update(cx, |input, cx| {
                            if input
                                .model
                                .select_accessible(&selection_snapshot, anchor, cursor)
                            {
                                input.model.upstream = accessible_upstream(
                                    &selection_snapshot,
                                    &mapping,
                                    &selection.focus,
                                );
                                cx.notify();
                            }
                        });
                    }
                },
            )
            .on_a11y_action(
                gpui::AccessibleAction::ReplaceSelectedText,
                move |data, _, cx| {
                    if let Some(gpui::accesskit::ActionData::Value(value)) = data {
                        let _ = replacement_target.update(cx, |input, cx| {
                            input.model.replace(None, value, false, None);
                            cx.notify();
                        });
                    }
                },
            )
            .w_full()
            .key_context(INPUT_CONTEXT)
            .track_focus(&self.focus)
            .cursor(CursorStyle::IBeam)
            .border_1()
            .border_color(rgb(0xaab4bd))
            .rounded_md()
            .p_2()
            .bg(rgb(0xffffff))
            .when(self.compact, |input| {
                input
                    .border_color(rgb(0xf8f9fc))
                    .bg(rgb(0xf8f9fc))
                    .focus(|style| style.border_color(rgb(0x5e81ac)).bg(rgb(0xffffff)))
            })
            .text_color(rgb(0x18242b))
            .text_size(px(16.))
            .line_height(px(26.))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(|this, _: &Up, _, cx| this.vertical(false, false, cx)))
            .on_action(cx.listener(|this, _: &Down, _, cx| this.vertical(true, false, cx)))
            .on_action(cx.listener(|this, _: &SelectUp, _, cx| this.vertical(false, true, cx)))
            .on_action(cx.listener(|this, _: &SelectDown, _, cx| this.vertical(true, true, cx)))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::cancel))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_scroll_wheel(cx.listener(|input, event: &gpui::ScrollWheelEvent, _, cx| {
                if let (Some(layout), Some(bounds)) = (&input.layout, input.bounds) {
                    let delta = event.delta.pixel_delta(layout.line_height);
                    let next = (input.scroll_y - delta.y)
                        .max(px(0.))
                        .min((layout.height() - bounds.size.height).max(px(0.)));
                    if next != input.scroll_y {
                        input.scroll_y = next;
                        cx.stop_propagation();
                        cx.notify();
                    }
                }
            }))
            .child(InputLine {
                input: cx.entity(),
                accessible_runs,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_panics_retain_text_selection_and_composition() {
        let mut model = TextModel {
            text: "中文草稿".into(),
            anchor: 0,
            cursor: 6,
            marked: Some(0..6),
            ..Default::default()
        };
        for cut in [false, true] {
            model.copy_to_clipboard(cut, |_| panic!("injected clipboard write failure"));
            assert_eq!(model.text, "中文草稿");
            assert_eq!(model.selection(), 0..6);
            assert_eq!(model.marked, Some(0..6));
        }
        model.paste_from_clipboard(|| panic!("injected clipboard read failure"));
        assert_eq!(model.text, "中文草稿");
        assert_eq!(model.selection(), 0..6);
        assert_eq!(model.marked, Some(0..6));
    }

    #[test]
    fn clipboard_cut_copies_before_removal_and_paste_replaces_selection() {
        let mut model = TextModel {
            text: "中文草稿".into(),
            anchor: 0,
            cursor: 6,
            ..Default::default()
        };
        let mut copied = String::new();
        model.copy_to_clipboard(false, |text| copied = text);
        assert_eq!(copied, "中文");
        assert_eq!(model.text, "中文草稿");
        model.copy_to_clipboard(true, |text| assert_eq!(text, "中文"));
        assert_eq!(model.text, "草稿");
        model.paste_from_clipboard(|| Some(copied));
        assert_eq!(model.text, "中文草稿");
        model.anchor = 0;
        model.cursor = 6;
        model.paste_from_clipboard(|| None);
        assert_eq!(model.text, "中文草稿");
        model.paste_from_clipboard(|| Some("English".into()));
        assert_eq!(model.text, "English草稿");
    }

    fn geometry_fixture() -> InputLayout {
        let text = "a😀\n中b\n";
        let ranges = [(0..1, 1), (1..5, 6), (6..10, 11), (11..11, 11)];
        let rows = ranges
            .into_iter()
            .map(|(range, accessible_end)| {
                let value = &text[range.clone()];
                let mut line = ShapedLine::default();
                *line = std::sync::Arc::new(gpui::LineLayout {
                    width: px(value.chars().count() as f32 * 10.),
                    len: value.len(),
                    runs: vec![gpui::ShapedRun {
                        font_id: gpui::FontId(0),
                        glyphs: value
                            .char_indices()
                            .enumerate()
                            .map(|(column, (byte, _))| gpui::ShapedGlyph {
                                id: gpui::GlyphId(0),
                                position: point(px(column as f32 * 10.), px(0.)),
                                index: byte,
                                is_emoji: false,
                            })
                            .collect(),
                    }],
                    ..Default::default()
                });
                line.text = value.to_owned().into();
                InputRow {
                    range,
                    accessible_end,
                    line,
                }
            })
            .collect();
        InputLayout {
            text: text.to_owned(),
            rows,
            line_height: px(26.),
            failed: false,
        }
    }

    #[test]
    fn visual_rows_share_unicode_caret_and_pointer_geometry() {
        let layout = geometry_fixture();
        for byte in [0, 1, 5, 6, 9, 10, 11] {
            assert_eq!(layout.index(layout.position(byte)), byte);
        }
        assert_eq!(layout.position(1), point(px(0.), px(26.)));
        assert_eq!(layout.position(5), point(px(10.), px(26.)));
        assert_eq!(layout.position(11), point(px(0.), px(78.)));
        assert_eq!(layout.index(point(px(-100.), px(-100.))), 0);
        assert_eq!(layout.index(point(px(1000.), px(1000.))), 11);
        assert_eq!(layout.index(point(px(1000.), px(30.))), 5);
        assert_eq!(layout.height(), px(104.));
        let (byte, upstream) = layout.hit(point(px(100.), px(0.)));
        assert_eq!((byte, upstream), (1, true));
        assert_eq!(
            layout.position_with_affinity(byte, upstream),
            point(px(10.), px(0.))
        );
        assert_eq!(
            layout.position_with_affinity(byte, false),
            point(px(0.), px(26.))
        );
    }

    #[test]
    fn shaping_failure_retains_text_without_native_fallback_calls() {
        let layout = fallback_layout("中文\n😀", px(26.));
        assert!(layout.failed);
        assert_eq!(layout.text, "中文\n😀");
        assert_eq!(layout.rows[0].accessible_end, layout.text.len());
        assert_eq!(layout.position(layout.text.len()), point(px(0.), px(0.)));
    }

    #[test]
    fn accessible_rows_round_trip_wraps_newlines_empty_line_and_reverse_selection() {
        let layout = geometry_fixture();
        let rows: Vec<_> = layout
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                (
                    gpui::accesskit::NodeId(i as u64 + 1),
                    row.range.start..row.accessible_end,
                )
            })
            .collect();
        for (scalar, byte) in layout
            .text
            .char_indices()
            .map(|(byte, _)| byte)
            .chain([layout.text.len()])
            .enumerate()
        {
            let position =
                accessible_position(&layout.text, &rows, byte, false).expect("valid position");
            assert_eq!(
                accessible_scalar(&layout.text, &rows, &position),
                Some(scalar)
            );
        }
        assert_eq!(
            accessible_scalar(
                &layout.text,
                &rows,
                &gpui::accesskit::TextPosition {
                    node: gpui::accesskit::NodeId(999),
                    character_index: 0,
                }
            ),
            None
        );
        assert_eq!(
            accessible_scalar(
                &layout.text,
                &rows,
                &gpui::accesskit::TextPosition {
                    node: rows[0].0,
                    character_index: 2,
                }
            ),
            None
        );
        let mut model = TextModel {
            multiline: true,
            ..Default::default()
        };
        model.replace(None, &layout.text, false, None);
        assert!(model.select_accessible(&layout.text, 5, 1));
        assert_eq!(&model.text[model.selection()], "😀\n中b");
        assert!(model.anchor > model.cursor);
    }

    #[test]
    fn composition_runs_are_sliced_without_losing_byte_coverage() {
        let run = TextRun {
            len: 3,
            font: gpui::Font::default(),
            color: rgb(0).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = vec![
            run.clone(),
            TextRun {
                len: 7,
                ..run.clone()
            },
            TextRun { len: 2, ..run },
        ];
        assert_eq!(
            runs_for_range(&runs, 2..11)
                .iter()
                .map(|run| run.len)
                .collect::<Vec<_>>(),
            vec![1, 7, 1]
        );
        assert!(runs_for_range(&runs, 3..3).is_empty());
    }

    #[test]
    fn multiline_normalizes_crlf_and_preserves_ime_selection() {
        let mut model = TextModel {
            multiline: true,
            ..Default::default()
        };
        model.replace(None, "a\r\n中😀", true, Some(3..6));
        assert_eq!(model.text, "a\n中😀");
        assert_eq!(&model.text[model.selection()], "中😀");
        model.cancel_composition();
        assert_eq!(model.text, "");
    }

    #[test]
    fn cancelled_composition_restores_soft_wrap_affinity() {
        let mut model = TextModel {
            multiline: true,
            ..Default::default()
        };
        model.replace(None, "abcd", false, None);
        model.move_to(2, false);
        model.upstream = true;
        model.replace(None, "中", true, None);
        assert!(!model.upstream);
        model.cancel_composition();
        assert_eq!(model.text, "abcd");
        assert_eq!(model.cursor, 2);
        assert!(model.upstream);
    }

    #[test]
    fn accessible_soft_wrap_affinity_distinguishes_same_byte_on_two_rows() {
        let text = "ab\nc";
        let rows = vec![
            (gpui::accesskit::NodeId(1), 0..1),
            (gpui::accesskit::NodeId(2), 1..3),
            (gpui::accesskit::NodeId(3), 3..4),
        ];
        let before = accessible_position(text, &rows, 1, true).expect("row end");
        let after = accessible_position(text, &rows, 1, false).expect("row start");
        assert_eq!(before.node, rows[0].0);
        assert_eq!(after.node, rows[1].0);
        assert!(accessible_upstream(text, &rows, &before));
        assert!(!accessible_upstream(text, &rows, &after));
        assert_eq!(accessible_scalar(text, &rows, &before), Some(1));
        assert_eq!(accessible_scalar(text, &rows, &after), Some(1));
        let newline_end = gpui::accesskit::TextPosition {
            node: rows[1].0,
            character_index: 2,
        };
        assert!(!accessible_upstream(text, &rows, &newline_end));
    }

    #[test]
    fn multiline_vertical_movement_and_line_edges_are_unicode_safe() {
        let mut model = TextModel {
            multiline: true,
            ..Default::default()
        };
        model.replace(None, "a😀b\ne\u{301}z\n", false, None);
        model.move_to("a😀".len(), false);
        assert_eq!(model.line_range(model.cursor), 0.."a😀b".len());
        model.move_to(model.vertical_destination(true), false);
        assert_eq!(&model.text[..model.cursor], "a😀b\ne\u{301}z");
        model.move_to(model.vertical_destination(true), true);
        assert_eq!(&model.text[model.selection()], "\n");
        assert_eq!(model.vertical_destination(true), model.cursor);
    }

    #[test]
    fn accessibility_selection_rejects_stale_or_invalid_offsets() {
        let mut model = TextModel::default();
        model.replace(None, "a😀b", false, None);
        let before = model.selection();
        assert!(!model.select_accessible("old", 0, 1));
        assert!(!model.select_accessible("a😀b", 0, 99));
        assert_eq!(model.selection(), before);
        assert!(model.select_accessible("a😀b", 2, 1));
        assert_eq!(&model.text[model.selection()], "😀");
        assert!(model.anchor > model.cursor);
    }

    #[test]
    fn accessibility_selection_round_trips_scalar_boundaries() {
        let mut model = TextModel::default();
        model.replace(None, "ae\u{301}z", false, None);
        assert!(model.select_accessible("ae\u{301}z", 2, 3));
        assert_eq!(&model.text[model.selection()], "\u{301}");
        model.delete(false);
        assert_eq!(model.text, "aez");
    }

    #[test]
    fn accessibility_selection_handles_long_combining_sequences() {
        let text = format!("a{}z", "\u{301}".repeat(200));
        let mut model = TextModel::default();
        model.replace(None, &text, false, None);
        for start in 0..=202 {
            assert!(model.select_accessible(&text, start, 202));
            assert_eq!(model.text[..model.anchor].chars().count(), start);
            assert_eq!(model.text[..model.cursor].chars().count(), 202);
        }
    }

    #[test]
    fn finishing_composition_prevents_stale_escape_and_replacement() {
        let mut model = TextModel::default();
        model.replace(None, "original", false, None);
        model.replace(None, "中文", true, None);
        model.unmark();
        model.cancel_composition();
        assert_eq!(model.text, "original中文");
        model.replace(None, "next", false, None);
        assert_eq!(model.text, "original中文next");
        assert!(model.marked.is_none());
        assert!(model.before_composition.is_none());
    }

    #[test]
    fn composition_selection_is_relative_to_insert_not_old_buffer() {
        let mut model = TextModel::default();
        model.replace(None, "前😀后", false, None);
        model.replace(Some(1..3), "中文😀", true, Some(2..4));
        assert_eq!(model.text, "前中文😀后");
        assert_eq!(&model.text[model.selection()], "😀");
        assert_eq!(model.to_utf16(model.selection()), 3..5);
        model.replace(None, "确认", false, None);
        assert_eq!(model.text, "前确认后");
        assert_eq!(model.marked, None);
    }

    #[test]
    fn cancel_restores_original_selection_across_composition_updates() {
        let mut model = TextModel::default();
        model.replace(None, "ab", false, None);
        model.move_to(1, false);
        model.replace(None, "中", true, Some(1..1));
        model.replace(None, "中文", true, Some(2..2));
        model.cancel_composition();
        assert_eq!(model.text, "ab");
        assert_eq!(model.selection(), 1..1);
    }

    #[test]
    fn grapheme_movement_and_deletion_preserve_combining_sequences() {
        let mut model = TextModel::default();
        model.replace(None, "中e\u{301}👩‍💻", false, None);
        model.delete(true);
        assert_eq!(model.text, "中e\u{301}");
        model.delete(true);
        assert_eq!(model.text, "中");
        model.move_to(0, false);
        model.delete(false);
        assert_eq!(model.text, "");
    }

    #[test]
    fn utf16_ranges_clamp_surrogates_and_invalid_order_without_panicking() {
        assert_eq!(TextModel::from_utf16("a😀b", 2..99), 1..6);
        assert_eq!(
            TextModel::from_utf16("a😀b", std::ops::Range { start: 3, end: 1 }),
            1..5
        );
        assert_eq!(TextModel::from_utf16("a😀b", 2..2), 1..1);
    }

    #[test]
    fn unmark_commits_and_newlines_are_single_line() {
        let mut model = TextModel::default();
        model.replace(None, "a\r\nb", true, Some(4..4));
        model.unmark();
        model.cancel_composition();
        assert_eq!(model.text, "a  b");
        assert_eq!(model.selection(), 4..4);
    }

    #[test]
    fn selection_crosses_anchor_and_replacement_resets_direction() {
        let mut model = TextModel::default();
        model.replace(None, "abcd", false, None);
        model.move_to(2, false);
        model.move_to(0, true);
        assert_eq!(model.selection(), 0..2);
        model.move_to(4, true);
        assert_eq!(model.selection(), 2..4);
        model.replace(None, "X", false, None);
        assert_eq!(model.text, "abX");
        assert_eq!(model.anchor, model.cursor);
    }
}
