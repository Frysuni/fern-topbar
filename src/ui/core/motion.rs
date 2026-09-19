//! Opacity animations that never change widget measurements or allocation.

use relm4::gtk;
use relm4::gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

/// Fades a widget from transparent to opaque over 180 milliseconds.
///
/// Uses GTK frame-clock ticks and cubic ease-out without changing margins, size
/// requests or layout. The callback removes itself when the animation completes.
/// Call once when mounting a widget. For repeated mapping, use
/// [`animate_appearance`] instead, which cancels superseded callbacks.
/// The frame clock must run for the widget to reach full opacity.
pub fn reveal(widget: &impl IsA<gtk::Widget>) {
    let widget = widget.clone().upcast::<gtk::Widget>();

    fade(&widget, Rc::new(Cell::new(0)));
}

/// Fades in on each mapping without changing the widget's size or allocation.
///
/// Install once after construction. Unmapping cancels the current fade, so rapid
/// close/reopen cycles never leave competing frame callbacks. Respects GTK's
/// animation setting; disabled animations show the widget immediately.
pub fn animate_appearance(widget: &impl IsA<gtk::Widget>) {
    let widget = widget.as_ref();
    let generation = Rc::new(Cell::new(0_u64));

    widget.connect_map({
        let generation = generation.clone();

        move |widget| fade(widget, generation.clone())
    });

    widget.connect_unmap({
        let generation = generation.clone();

        move |widget| {
            generation.set(generation.get().wrapping_add(1));
            widget.set_opacity(1.0);
        }
    });

    if widget.is_mapped() {
        fade(widget, generation);
    }
}

/// Starts one frame-clock fade and discards callbacks from previous mappings.
fn fade(widget: &gtk::Widget, generation: Rc<Cell<u64>>) {
    let current = generation.get().wrapping_add(1);

    generation.set(current);

    if !widget.settings().is_gtk_enable_animations() {
        widget.set_opacity(1.0);
        return;
    }

    widget.set_opacity(0.0);

    let started = Instant::now();

    widget.add_tick_callback(move |widget, _| {
        if generation.get() != current {
            return gtk::glib::ControlFlow::Break;
        }

        let progress = (started.elapsed().as_secs_f64() / 0.18).min(1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);

        widget.set_opacity(eased);

        if progress >= 1.0 {
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[gtk::test]
    fn appearance_fades_cancel_on_unmap_and_preserve_measurements() {
        let window = gtk::Window::new();
        let label = gtk::Label::new(Some("Stable layout"));
        let settings = label.settings();
        let enabled = settings.is_gtk_enable_animations();

        settings.set_gtk_enable_animations(true);
        window.set_child(Some(&label));

        let measured = label.measure(gtk::Orientation::Vertical, -1);

        animate_appearance(&label);

        window.present();

        assert_eq!(label.opacity(), 0.0);

        window.set_visible(false);

        assert_eq!(label.opacity(), 1.0);

        window.present();

        assert_eq!(label.opacity(), 0.0);

        let context = gtk::glib::MainContext::default();

        for _ in 0..60 {
            while context.pending() {
                context.iteration(false);
            }

            thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(label.opacity(), 1.0);
        assert_eq!(label.measure(gtk::Orientation::Vertical, -1), measured);

        window.set_visible(false);
        settings.set_gtk_enable_animations(false);
        window.present();

        assert_eq!(label.opacity(), 1.0);

        window.destroy();
        settings.set_gtk_enable_animations(enabled);
    }
}
