use super::CalendarDay;
use crate::ui::core::Button;
use relm4::factory::positions::GridPosition;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque, Position};
use relm4::gtk;
use relm4::gtk::prelude::*;

const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

pub struct CalendarCell {
    content: CellContent,
}

pub enum CellContent {
    Weekday(&'static str),
    Day(CalendarDay),
}

#[relm4::factory(pub)]
impl FactoryComponent for CalendarCell {
    type Init = CellContent;
    type Input = ();
    type Output = DynamicIndex;
    type CommandOutput = ();
    type ParentWidget = gtk::Grid;

    view! {
        #[root]
        gtk::Box {
            gtk::Label {
                add_css_class: "topbar-calendar-weekday",
                #[watch]
                set_label: self.weekday().unwrap_or(""),
                #[watch]
                set_visible: self.weekday().is_some(),
            },
            #[template]
            Button {
                #[watch]
                set_css_classes: self.day_classes(),
                #[watch]
                set_label: self.day().map_or("", |day| day.label.as_str()),
                #[watch]
                set_tooltip_text: self.day().map(|day| day.tooltip.as_str()),
                #[watch]
                set_visible: self.day().is_some(),
                connect_clicked[sender, index] => move |_| {
                    let _ = sender.output(index.clone());
                },
            },
        }
    }

    fn init_model(
        content: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { content }
    }
}

impl Position<GridPosition, DynamicIndex> for CalendarCell {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();

        GridPosition {
            column: (index % 7) as i32,
            row: (index / 7) as i32,
            width: 1,
            height: 1,
        }
    }
}

impl CalendarCell {
    fn weekday(&self) -> Option<&str> {
        match &self.content {
            CellContent::Weekday(weekday) => Some(weekday),
            CellContent::Day(_) => None,
        }
    }

    fn day(&self) -> Option<&CalendarDay> {
        match &self.content {
            CellContent::Weekday(_) => None,
            CellContent::Day(day) => Some(day),
        }
    }

    fn day_classes(&self) -> &'static [&'static str] {
        match self.day() {
            Some(day) => match (day.in_month, day.today, day.selected) {
                (true, true, true) => &["topbar-calendar-day", "today", "selected"],
                (true, true, false) => &["topbar-calendar-day", "today"],
                (true, false, true) => &["topbar-calendar-day", "selected"],
                (true, false, false) => &["topbar-calendar-day"],
                (false, true, true) => &["topbar-calendar-day", "other-month", "today", "selected"],
                (false, true, false) => &["topbar-calendar-day", "other-month", "today"],
                (false, false, true) => &["topbar-calendar-day", "other-month", "selected"],
                (false, false, false) => &["topbar-calendar-day", "other-month"],
            },
            None => &[],
        }
    }

    fn replace(&mut self, content: CellContent) {
        self.content = content;
    }

    pub fn date(&self) -> Option<chrono::NaiveDate> {
        self.day().map(|day| day.date)
    }
}

pub fn sync(factory: &mut FactoryVecDeque<CalendarCell>, days: &[CalendarDay]) {
    let cells = WEEKDAYS
        .into_iter()
        .map(CellContent::Weekday)
        .chain(days.iter().cloned().map(CellContent::Day));

    let mut cells = cells.collect::<Vec<_>>();
    let mut factory = factory.guard();

    while factory.len() > cells.len() {
        factory.pop_back();
    }

    for (index, content) in cells.drain(..).enumerate() {
        if let Some(cell) = factory.get_mut(index) {
            cell.replace(content);
        } else {
            factory.push_back(content);
        }
    }
}
