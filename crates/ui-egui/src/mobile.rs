//! The phone layout: a touch-first shell that replaces the desktop title bar, tab strip, ribbon
//! and status bar when the window is narrow.
//!
//! A phone cannot fit the desktop's 38 px title bar, 30 px tab strip and 94 px ribbon at once —
//! together they ate most of a 360 × 700 screen, and the 22–24 px buttons were hard to hit. So on
//! a narrow window the app shows:
//!
//! - a 52 px top bar with File, Save, Undo, Redo and Search, each a 52 × 48 touch target;
//! - the document, full width, with no panes or rulers;
//! - a 58 px bottom bar of ribbon tabs; tapping one shows that tab's controls in a sheet above
//!   the bar, the same controls the desktop ribbon draws ([`crate::ribbon::tab_content`]).
//!
//! The layout follows the window's width, not the platform, so a small desktop window gets the
//! same usable shell.

use egui::{Align, Align2, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::theme::{Tokens, medium, regular, semibold};
use crate::{WordApp, icons};

/// Width below which the desktop chrome no longer fits and the phone shell takes over.
const PHONE_WIDTH: f32 = 720.0;

/// A tab's ribbon sheet is open or closed; kept in egui's per-context memory.
fn sheet_id() -> egui::Id {
    egui::Id::new("wordcraft_mobile_sheet")
}

/// Whether the window is phone-shaped.
pub(crate) fn touch(ctx: &egui::Context) -> bool {
    ctx.content_rect().width() < PHONE_WIDTH
}

/// Make the shared widgets finger-sized while the window is phone-shaped. Called once per frame
/// from [`WordApp::logic`](crate::WordApp) after the theme is applied, so it wins over the
/// desktop metrics and is restored when the window grows again. It writes only when the size is
/// not already what this window needs, so a phone does not rebuild its styles every frame.
pub(crate) fn apply_touch_style(ctx: &egui::Context) {
    let phone = touch(ctx);
    let (interact, padding, menu) = if phone { (vec2(34.0, 34.0), vec2(9.0, 5.0), 8) } else { (vec2(24.0, 22.0), vec2(8.0, 3.0), 6) };
    if ctx.style_of(egui::Theme::Dark).spacing.interact_size == interact {
        return;
    }
    ctx.all_styles_mut(|s| {
        s.spacing.interact_size = interact;
        s.spacing.button_padding = padding;
        s.spacing.menu_margin = egui::Margin::same(menu);
    });
}

/// Draw the whole phone window: the File page, or the top bar, document, tab bar and sheet.
pub(crate) fn paint(app: &mut WordApp, ui: &mut Ui, ctx: &egui::Context, t: Tokens) {
    if app.ui.backstage {
        crate::backstage::show(app, ui);
        return;
    }
    top_bar(app, ui, t);
    if app.has_document() {
        bottom_bar(app, ui, t);
        sheet(app, ui, ctx, t);
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |ui| {
            crate::canvas::show(app, ui);
        });
    } else {
        // No document: the top bar's File page is the way to make or open one.
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.canvas)).show(ui, |_| {});
    }
    toast(app, ctx, t);
}

/// The status bar is desktop chrome; on a phone its message appears as a short-lived toast above
/// the tab bar, so "Saved …" and "Couldn't write …" are still seen.
fn toast(app: &WordApp, ctx: &egui::Context, t: Tokens) {
    let Some((msg, at)) = &app.status_msg else { return };
    if crate::now_ms() - at > 6000.0 {
        return;
    }
    let width = (ctx.content_rect().width() - 24.0).max(80.0);
    let at = pos2(12.0, ctx.content_rect().height() - 132.0);
    egui::Area::new(egui::Id::new("wordcraft_mobile_toast")).order(egui::Order::Foreground).fixed_pos(at).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.set_max_width(width);
            ui.label(egui::RichText::new(msg).font(regular(12.0)).color(t.text));
        });
    });
}

/// A touch target with a drawn icon and a small label under it.
fn touch_button(ui: &mut Ui, t: Tokens, icon: &str, label: &str, enabled: bool) -> bool {
    let label = tl!(label).to_string();
    let (r, resp) = ui.allocate_exact_size(vec2(52.0, 48.0), if enabled { Sense::click() } else { Sense::hover() });
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r.shrink2(vec2(2.0, 4.0)), 6.0, t.hover);
    }
    let (c, a) = if enabled { (t.icon, t.accent) } else { (t.text_disabled, t.text_disabled) };
    icons::paint(ui.painter(), Rect::from_center_size(pos2(r.center().x, r.min.y + 16.0), vec2(22.0, 22.0)), icon, c, a);
    ui.painter().text(
        pos2(r.center().x, r.max.y - 11.0),
        Align2::CENTER_CENTER,
        &label,
        regular(10.0),
        if enabled { t.text } else { t.text_disabled },
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &label));
    enabled && resp.clicked()
}

/// File, Save, Undo, Redo and Search: everything a phone needs without opening the ribbon.
fn top_bar(app: &mut WordApp, ui: &mut Ui, t: Tokens) {
    egui::Panel::top("mobile_top")
        .exact_size(52.0)
        .frame(
            egui::Frame::NONE
                .fill(t.title_bar)
                .inner_margin(egui::Margin { left: 2, right: 2, top: 0, bottom: 0 })
                .stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                if touch_button(ui, t, "file", "File", true) {
                    app.ui.backstage = true;
                    app.ui.backstage_page = "home".into();
                }
                let has_doc = app.has_document();
                if touch_button(ui, t, "save", "Save", has_doc) {
                    let _ = app.run("file.save", serde_json::json!({}));
                }
                if touch_button(ui, t, "undo", "Undo", has_doc && app.session.can_undo()) {
                    let _ = app.run("edit.undo", serde_json::json!({}));
                }
                if touch_button(ui, t, "redo", "Redo", has_doc && app.session.can_redo()) {
                    let _ = app.run("edit.redo", serde_json::json!({}));
                }
                ui.with_layout(crate::rtl::far_side(ui, Align::Center), |ui| {
                    if touch_button(ui, t, "search", "Search", has_doc) {
                        let _ = app.run("tools.search", serde_json::json!({}));
                    }
                });
            });
        });
}

/// The ribbon tabs, as pills a thumb can hit. A tap opens that tab's sheet.
fn bottom_bar(app: &mut WordApp, ui: &mut Ui, t: Tokens) {
    let tabs = crate::ribbon::visible_tabs(app);
    let active = app.ui.tab.clone();
    egui::Panel::bottom("mobile_tabs")
        .exact_size(58.0)
        .frame(
            egui::Frame::NONE
                .fill(t.tab_strip)
                .inner_margin(egui::Margin { left: 4, right: 4, top: 4, bottom: 6 })
                .stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| {
            egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    for tab in tabs.iter().filter(|tab| tab.as_str() != "File") {
                        let shown = tl!(tab.as_str()).to_string();
                        let w = ui.ctx().fonts_mut(|f| f.layout_no_wrap(shown.clone(), medium(12.0), t.text).size().x) + 26.0;
                        let (r, resp) = ui.allocate_exact_size(vec2(w.max(58.0), 44.0), Sense::click());
                        let on = &active == tab;
                        if on {
                            ui.painter().rect_filled(r, 20.0, t.accent);
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, 20.0, t.hover);
                        }
                        ui.painter().text(
                            r.center(),
                            Align2::CENTER_CENTER,
                            &shown,
                            if on { semibold(12.0) } else { medium(12.0) },
                            if on { t.on_accent } else { t.text },
                        );
                        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, &shown));
                        if resp.clicked() {
                            let _ = app.run("ui.tab", serde_json::json!({"tab": tab}));
                            ui.data_mut(|d| d.insert_temp(sheet_id(), true));
                        }
                    }
                });
            });
        });
}

/// The active tab's controls, in a sheet above the tab bar. It scrolls sideways like the ribbon;
/// the chevron closes it so the page can use the whole screen, and tapping a tab reopens it.
fn sheet(app: &mut WordApp, ui: &mut Ui, ctx: &egui::Context, t: Tokens) {
    if !ctx.data(|d| d.get_temp::<bool>(sheet_id())).unwrap_or(true) {
        return;
    }
    let height = (ctx.content_rect().height() * 0.42).clamp(150.0, 330.0);
    egui::Panel::bottom("mobile_sheet")
        .exact_size(height)
        .frame(
            egui::Frame::NONE.fill(t.ribbon).inner_margin(egui::Margin { left: 6, right: 6, top: 2, bottom: 4 }).stroke(Stroke::new(1.0, t.border)),
        )
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let name = tl!(app.ui.tab.as_str()).to_string();
                ui.label(egui::RichText::new(name).font(semibold(13.0)).color(t.accent_text));
                // The document's name, so it is visible even though the desktop title bar is gone.
                let title = app.title_stem();
                ui.label(egui::RichText::new(title).font(regular(11.0)).color(t.text_dim));
                ui.with_layout(crate::rtl::far_side(ui, Align::Center), |ui| {
                    let hide = tl!("Hide the ribbon");
                    if ui.add(egui::Button::new(egui::RichText::new("▾").font(regular(16.0))).frame(false)).on_hover_text(hide).clicked() {
                        ui.data_mut(|d| d.insert_temp(sheet_id(), false));
                    }
                });
            });
            let w = ui.available_width();
            crate::ribbon_fit::begin(ctx, &app.ui.tab, w);
            egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                // The same row shape the desktop ribbon uses, so the groups get the same width
                // instead of an unbounded one inside a horizontal scroll area.
                crate::rtl::horizontal_top(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                    crate::ribbon::tab_content(app, ui);
                });
            });
            crate::ribbon_fit::end(ctx);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run one frame in a window of this width; the body runs inside it.
    fn frame(ctx: &egui::Context, width: f32, mut body: impl FnMut(&mut Ui)) {
        let input = egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(width, 700.0))), ..Default::default() };
        ctx.run_ui(input, |ui| body(ui)).drop_without_applying_deltas();
    }

    #[test]
    fn a_narrow_window_is_a_phone_and_a_wide_one_is_not() {
        let ctx = egui::Context::default();
        for (width, phone) in [(360.0, true), (719.0, true), (720.0, false), (1280.0, false)] {
            let mut seen = None;
            frame(&ctx, width, |ui| seen = Some(touch(ui.ctx())));
            assert_eq!(seen, Some(phone), "width {width}");
        }
    }

    #[test]
    fn touch_metrics_follow_the_window_and_restore_the_desktop_ones() {
        let ctx = egui::Context::default();
        let mut phone = None;
        frame(&ctx, 360.0, |ui| {
            apply_touch_style(ui.ctx());
            phone = Some(ui.ctx().style_of(egui::Theme::Light).spacing.interact_size);
        });
        assert_eq!(phone, Some(vec2(34.0, 34.0)));
        let mut desktop = None;
        frame(&ctx, 1280.0, |ui| {
            apply_touch_style(ui.ctx());
            desktop = Some(ui.ctx().style_of(egui::Theme::Light).spacing.interact_size);
        });
        assert_eq!(desktop, Some(vec2(24.0, 22.0)));
    }
}
