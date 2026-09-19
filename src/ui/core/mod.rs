//! Reusable GTK building blocks, independent of feature state and backends.
//!
//! Configured widgets implement Relm4's `WidgetTemplate`; templates accessed
//! outside the DSL expose their GTK root through `widget()`. Use `#[template]`
//! in `view!`; Relm4 generates the template's required framework plumbing.
//!
//! The panel creates one [`PopoverScope`] scope and passes its clones to children.
//! Constructing a menu does not attach it to that scope: register it after its
//! popover has been assigned and retain the returned guard. All widgets and scopes belong to GTK's UI thread.

#![deny(missing_docs)]

mod buttons;
mod disclosure;
mod layout;
/// Layout-neutral appearance animations for widgets.
pub mod motion;
mod popover;
mod popover_scope;
mod scale;
mod scaling;

pub use buttons::{
    ActionButton, ActionToggleButton, Button, MenuButtonStyle, PanelIconButton, PanelMenuButton,
};
pub use disclosure::Disclosure;
pub use layout::{FeatureDivider, StatusBox};
pub use popover::{MenuPopover, PopoverStyle};
pub use popover_scope::{PopoverScope, PopupId, PopupRegistration};
pub use scale::{PercentScale, PercentValue};
pub use scaling::UiScale;

#[cfg(test)]
mod tests {
    use super::*;
    use relm4::WidgetTemplate;
    use relm4::gtk;
    use relm4::gtk::prelude::*;

    #[gtk::test]
    fn templates_preserve_shared_widget_contracts() {
        let action = ActionButton::init(());
        let toggle = ActionToggleButton::init(());

        assert!(action.widget().has_frame());
        assert!(toggle.widget().has_frame());
        assert!(action.widget().has_css_class("topbar-action-button"));
        assert!(toggle.widget().has_css_class("topbar-action-button"));
        assert!(!action.widget().has_css_class("flat"));
        assert!(!toggle.widget().has_css_class("flat"));
        assert!(!action.widget().gets_focus_on_click());
        assert!(!toggle.widget().gets_focus_on_click());
        assert!(action.widget().is_focusable());
        assert!(toggle.widget().is_focusable());

        let plain = Button::init(());

        assert!(!plain.widget().gets_focus_on_click());
        assert!(plain.widget().is_focusable());

        let button_template = PanelMenuButton::init(MenuButtonStyle::Icon);
        let button = button_template.widget();

        assert!(button.has_css_class("topbar-feature-button"));
        assert!(!button.has_frame());
        assert!(!button.has_css_class("topbar-labeled-button"));
        assert!(button.popover().is_none());
        assert!(!button.gets_focus_on_click());

        let labeled_template = PanelMenuButton::init(MenuButtonStyle::Labeled);
        let labeled = labeled_template.widget();

        assert!(labeled.has_css_class("topbar-feature-button"));
        assert!(labeled.has_css_class("topbar-labeled-button"));
        assert!(!labeled.has_frame());
        assert!(!labeled.gets_focus_on_click());

        let icon_template = PanelIconButton::init(());
        let icon = icon_template.widget();

        assert!(icon.has_css_class("topbar-icon-button"));
        assert!(icon.child().is_none());
        assert!(!icon.gets_focus_on_click());
        assert!(icon.is_focusable());

        let menu_template = MenuPopover::init(PopoverStyle::Menu);
        let menu = menu_template.widget();
        let mixer_template = MenuPopover::init(PopoverStyle::Mixer);
        let mixer = mixer_template.widget();

        assert!(menu.has_css_class("topbar-menu"));
        assert!(mixer.has_css_class("topbar-mixer"));
        assert!(!mixer.has_css_class("topbar-menu"));

        for popover in [&menu, &mixer] {
            assert!(!popover.has_arrow());
            assert_eq!(popover.position(), gtk::PositionType::Bottom);
            assert_eq!(popover.offset(), (0, 12));
            assert!(popover.is_autohide());
            assert!(popover.parent().is_none());
            assert!(popover.child().is_none());
        }

        for minimum in [0, 1] {
            let scale_template = PercentScale::init(minimum);
            let scale = scale_template.widget();

            assert_eq!(scale.orientation(), gtk::Orientation::Horizontal);
            assert_eq!(scale.width_request(), 148);
            assert!(!scale.draws_value());

            let adjustment = scale.adjustment();

            assert_eq!(adjustment.lower(), f64::from(minimum));
            assert_eq!(adjustment.upper(), 100.0);
            assert_eq!(adjustment.step_increment(), 1.0);
            assert_eq!(adjustment.page_increment(), 10.0);
        }

        let value_template = PercentValue::init(());
        let value = &value_template.root;

        assert!(value.has_css_class("topbar-volume-value"));
        assert_eq!(value.width_chars(), 4);
        assert_eq!(value.max_width_chars(), 4);
        assert_eq!(value.xalign(), 1.0);
        assert!(value.label().is_empty());

        let status_template = StatusBox::init(());
        let status = status_template.widget();

        assert!(status.has_css_class("topbar-status"));
        assert_eq!(status.orientation(), gtk::Orientation::Horizontal);
        assert_eq!(status.spacing(), 4);
        assert!(status.first_child().is_none());

        let divider_template = FeatureDivider::init(());
        let divider = divider_template.widget();

        assert!(divider.has_css_class("topbar-feature-divider"));
        assert_eq!(divider.orientation(), gtk::Orientation::Vertical);
        assert_eq!(divider.width_request(), 1);
        assert_eq!(divider.height_request(), 16);
        assert_eq!(divider.valign(), gtk::Align::Center);
        assert!(divider.get_visible());
    }

    #[gtk::test]
    fn buttons_remain_keyboard_navigable_and_activatable() {
        let plain = Button::init(());
        let action = ActionButton::init(());
        let toggle = ActionToggleButton::init(());
        let icon = PanelIconButton::init(());
        let menu = PanelMenuButton::init(MenuButtonStyle::Labeled);
        menu.widget().set_label("Menu");

        let popover = MenuPopover::init(PopoverStyle::Menu);

        popover
            .widget()
            .set_child(Some(&gtk::Label::new(Some("Menu content"))));
        menu.widget().set_popover(Some(popover.widget()));

        let _popup = PopoverScope::default().register_button(menu.widget());

        let buttons: [&gtk::Widget; 5] = [
            plain.as_ref(),
            action.as_ref(),
            toggle.as_ref(),
            icon.as_ref(),
            menu.as_ref(),
        ];

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        for button in buttons {
            container.append(button);
        }

        let window = gtk::Window::new();
        window.set_child(Some(&container));
        window.present();

        let main_loop = gtk::glib::MainLoop::new(None, false);
        let quit = main_loop.clone();

        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            quit.quit()
        });
        main_loop.run();

        assert!(plain.widget().grab_focus());

        for button in &buttons[1..] {
            assert!(window.child_focus(gtk::DirectionType::TabForward));

            let focused = GtkWindowExt::focus(&window).unwrap();

            assert!(
                focused == **button || focused.is_ancestor(*button),
                "expected {button:?}, focused {focused:?}"
            );
        }

        for button in buttons[..4].iter().rev() {
            assert!(window.child_focus(gtk::DirectionType::TabBackward));

            let focused = GtkWindowExt::focus(&window).unwrap();

            assert!(focused == **button || focused.is_ancestor(*button));
        }

        let activations = std::rc::Rc::new(std::cell::Cell::new(0));

        for button in [plain.widget(), action.widget(), icon.widget()] {
            let activations = activations.clone();

            button.connect_clicked(move |_| activations.set(activations.get() + 1));
        }

        for button in buttons {
            assert!(button.grab_focus());
            assert!(button.activate());

            let main_loop = gtk::glib::MainLoop::new(None, false);
            let quit = main_loop.clone();

            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                quit.quit()
            });
            main_loop.run();
        }

        assert_eq!(activations.get(), 3);
        assert!(toggle.widget().is_active());
        assert!(menu.widget().is_active());
        assert!(popover.widget().get_visible());

        window.destroy();
    }
}
