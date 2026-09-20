use crate::ui::{
    core::{
        Button, MenuButtonStyle, MenuPopover, PanelMenuButton, PopoverScope, PopoverStyle,
        PopupRegistration,
    },
    icon_names,
};
use chrono::{DateTime, Datelike, Duration as DateDuration, Local, NaiveDate};
use day::CalendarCell;
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::prelude::*;

mod day;

pub struct Clock {
    _popup: Option<PopupRegistration>,
    now: DateTime<Local>,
    month: CalendarMonth,
    selected: NaiveDate,
    view: Option<ClockView>,
    days: FactoryVecDeque<CalendarCell>,
}

pub struct ClockInit {
    pub popovers: PopoverScope,
    pub now: DateTime<Local>,
}

#[derive(Debug)]
pub enum Input {
    Tick(DateTime<Local>),
    PreviousMonth,
    NextMonth,
    Today,
    SelectCell(DynamicIndex),
}

#[relm4::component(pub)]
impl Component for Clock {
    type Init = ClockInit;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        #[template]
        PanelMenuButton(MenuButtonStyle::Labeled) {
            add_css_class: "topbar-time",
            set_direction: gtk::ArrowType::None,
            #[watch]
            set_label: model.view.as_ref().map_or("", |view| view.time.as_str()),
            #[wrap(Some)]
            #[template]
            set_popover = &MenuPopover(PopoverStyle::Menu) {
                gtk::Box {
                    add_css_class: "topbar-calendar",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    gtk::Box {
                        add_css_class: "topbar-calendar-header",
                        set_spacing: 6,
                        gtk::Label {
                            add_css_class: "topbar-calendar-title",
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                            #[watch]
                            set_label: model.view.as_ref().map_or("", |view| view.month.as_str()),
                        },
                        #[template]
                        Button {
                            add_css_class: "topbar-calendar-nav",
                            set_icon_name: icon_names::CHEVRON_LEFT,
                            set_tooltip_text: Some("Previous month"),
                            connect_clicked => Input::PreviousMonth,
                        },
                        #[template]
                        Button {
                            add_css_class: "topbar-calendar-nav",
                            set_icon_name: icon_names::CHEVRON_RIGHT,
                            set_tooltip_text: Some("Next month"),
                            connect_clicked => Input::NextMonth,
                        },
                    },
                    #[local_ref]
                    days_grid -> gtk::Grid {
                        add_css_class: "topbar-calendar-grid",
                        set_row_spacing: 3,
                        set_column_spacing: 3,
                        set_halign: gtk::Align::Center,
                    },
                    #[template]
                    Button {
                        add_css_class: "topbar-calendar-today",
                        set_label: "Back to today",
                        #[watch]
                        set_visible: model.view.as_ref().is_some_and(|view| view.show_today),
                        connect_clicked => Input::Today,
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ClockInit { popovers, now } = init;
        let today = now.date_naive();
        let days = FactoryVecDeque::builder()
            .launch_default()
            .forward(sender.input_sender(), Input::SelectCell);

        let mut model = Self {
            _popup: None,
            now,
            month: CalendarMonth::new(today),
            selected: today,
            view: None,
            days,
        };

        model.view = Some(model.view());
        day::sync(&mut model.days, &model.view.as_ref().unwrap().days);

        let days_grid = model.days.widget();
        let widgets = view_output!();

        model._popup = Some(popovers.register_button(root.widget()));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Input::Tick(now) => self.apply_clock_tick(now),
            Input::PreviousMonth => self.month = self.month.previous(),
            Input::NextMonth => self.month = self.month.next(),
            Input::Today => self.select_date(self.now.date_naive()),
            Input::SelectCell(index) => {
                if let Some(date) = self
                    .days
                    .get(index.current_index())
                    .and_then(CalendarCell::date)
                {
                    self.select_date(date);
                }
            }
        }

        self.view = Some(self.view());
        day::sync(&mut self.days, &self.view.as_ref().unwrap().days);
        self.update_view(widgets, sender);
    }
}

#[derive(Clone)]
pub struct CalendarDay {
    date: NaiveDate,
    label: String,
    tooltip: String,
    in_month: bool,
    today: bool,
    selected: bool,
}

struct ClockView {
    time: String,
    month: String,
    days: Vec<CalendarDay>,
    show_today: bool,
}

impl Clock {
    fn apply_clock_tick(&mut self, now: DateTime<Local>) {
        let previous_today = self.now.date_naive();
        let follow_today =
            self.selected == previous_today && self.month == CalendarMonth::new(previous_today);

        self.now = now;

        if follow_today {
            self.select_date(self.now.date_naive());
        }
    }

    fn select_date(&mut self, date: NaiveDate) {
        self.selected = date;
        self.month = CalendarMonth::new(date);
    }
}

impl Clock {
    fn view(&self) -> ClockView {
        clock_view(&self.now, self.month, self.selected)
    }
}

fn clock_view(now: &DateTime<Local>, month: CalendarMonth, selected: NaiveDate) -> ClockView {
    let today = now.date_naive();
    let start = month.0 - DateDuration::days(month.0.weekday().num_days_from_monday().into());
    let next = month.next();
    let days_in_month = (next.0 - month.0).num_days() as usize;
    let weeks = (month.0.weekday().num_days_from_monday() as usize + days_in_month).div_ceil(7);
    let days = (0..weeks * 7)
        .map(|index| {
            let date = start + DateDuration::days(index as i64);

            CalendarDay {
                date,
                label: date.day().to_string(),
                tooltip: date.format("%A, %B %-d, %Y").to_string(),
                in_month: date.month() == month.0.month() && date.year() == month.0.year(),
                today: date == today,
                selected: date == selected,
            }
        })
        .collect();

    ClockView {
        time: now.format("%A, %B %-d, %Y · %H:%M").to_string(),
        month: month.0.format("%B %Y").to_string(),
        days,
        show_today: month != CalendarMonth::new(today) || selected != today,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CalendarMonth(NaiveDate);

impl CalendarMonth {
    fn new(date: NaiveDate) -> Self {
        Self(
            NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
                .expect("valid date has a first day"),
        )
    }

    fn next(self) -> Self {
        let (year, month) = if self.0.month() == 12 {
            (self.0.year() + 1, 1)
        } else {
            (self.0.year(), self.0.month() + 1)
        };

        Self(NaiveDate::from_ymd_opt(year, month, 1).expect("calendar month is in range"))
    }

    fn previous(self) -> Self {
        let (year, month) = if self.0.month() == 1 {
            (self.0.year() - 1, 12)
        } else {
            (self.0.year(), self.0.month() - 1)
        };

        Self(NaiveDate::from_ymd_opt(year, month, 1).expect("calendar month is in range"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_aligns_leap_february_and_crosses_year_boundary() {
        let selected = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        let view = clock_view(&Local::now(), CalendarMonth::new(selected), selected);

        assert_eq!(
            view.days.first().unwrap().date,
            NaiveDate::from_ymd_opt(2024, 1, 29).unwrap()
        );
        assert!(view.days.iter().any(|day| {
            day.date == NaiveDate::from_ymd_opt(2024, 2, 29).unwrap() && day.in_month
        }));
        assert_eq!(
            CalendarMonth::new(NaiveDate::from_ymd_opt(2024, 12, 1).unwrap()).next(),
            CalendarMonth::new(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        );
    }
}
