use crate::backend::notifications::{Event, Notification, Urgency};
use std::time::{Duration, Instant};

const HISTORY_LIMIT: usize = 50;
const PREVIEW_LIMIT: usize = 3;

#[derive(Debug)]
pub struct Display {
    pub items: Vec<Notification>,
    pub hidden: usize,
    pub expanded: bool,
}

#[derive(Default)]
pub struct Center {
    history: Vec<Notification>,
    previews: Vec<Preview>,
    expanded: bool,
    hovered: bool,
    popup_open: bool,
}

impl Center {
    pub fn apply(&mut self, event: Event) {
        self.apply_at(event, Instant::now());
    }

    fn apply_at(&mut self, event: Event, now: Instant) {
        match event {
            Event::Added(item, timeout) => {
                let previous = self.history.iter().find(|previous| previous.id == item.id);
                let showing = self
                    .previews
                    .iter()
                    .any(|preview| preview.notification.id == item.id);
                // Replacing a retained notification updates it quietly after its
                // preview disappears. Escalation or explicit renewed attention
                // can surface it again; quiet alert updates cannot.
                let show_preview = showing
                    || item.request_attention
                    || previous.is_none_or(|previous| item.urgency > previous.urgency);

                self.history.retain(|previous| previous.id != item.id);
                self.history.push(item.clone());

                self.previews
                    .retain(|preview| preview.notification.id != item.id);

                if show_preview {
                    let timer = expiration_timer(item.urgency, timeout, now, self.is_paused());

                    self.previews.push(Preview {
                        notification: item,
                        timer,
                    });
                }

                // Critical alerts remain in history until explicitly closed,
                // even when they alone exceed the history budget.
                while self.history.len() > HISTORY_LIMIT {
                    let Some(index) = self
                        .history
                        .iter()
                        .position(|item| item.urgency != Urgency::Critical)
                    else {
                        break;
                    };

                    let removed = self.history.remove(index);

                    self.previews
                        .retain(|preview| preview.notification.id != removed.id);
                }

                while self.previews.len() > PREVIEW_LIMIT {
                    let Some(index) = self
                        .previews
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, preview)| preview.notification.urgency)
                        .map(|(index, _)| index)
                    else {
                        break;
                    };

                    self.previews.remove(index);
                }
            }
            Event::Closed(id) => self.dismiss(id),
        }
    }

    pub fn dismiss(&mut self, id: u32) {
        let was_paused = self.is_paused();

        self.history.retain(|item| item.id != id);
        self.previews
            .retain(|preview| preview.notification.id != id);

        if self.history.is_empty() {
            self.expanded = false;
        }

        self.update_pause(was_paused, self.is_paused(), Instant::now());
    }

    pub fn hide_previews(&mut self, ids: &[u32]) {
        self.previews
            .retain(|preview| !ids.contains(&preview.notification.id));
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.previews.clear();
        self.expanded = false;
    }

    pub fn expand(&mut self, expanded: bool) {
        self.expand_at(expanded, Instant::now());
    }

    fn expand_at(&mut self, expanded: bool, now: Instant) {
        let was_paused = self.is_paused();

        self.expanded = expanded && !self.history.is_empty();
        self.update_pause(was_paused, self.is_paused(), now);
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.set_hovered_at(hovered, Instant::now());
    }

    fn set_hovered_at(&mut self, hovered: bool, now: Instant) {
        let was_paused = self.is_paused();

        self.hovered = hovered && !self.popup_open;
        self.update_pause(was_paused, self.is_paused(), now);
    }

    pub fn set_popup_open(&mut self, open: bool) {
        self.set_popup_open_at(open, Instant::now());
    }

    fn set_popup_open_at(&mut self, open: bool, now: Instant) {
        let was_paused = self.is_paused();

        self.popup_open = open;

        if open {
            // Hiding the toast window may not produce a pointer leave event.
            self.hovered = false;
        }

        self.update_pause(was_paused, self.is_paused(), now);
    }

    fn is_paused(&self) -> bool {
        self.hovered || self.expanded || self.popup_open
    }

    fn update_pause(&mut self, was_paused: bool, is_paused: bool, now: Instant) {
        if was_paused == is_paused {
            return;
        }

        for preview in &mut self.previews {
            preview.timer = match (is_paused, preview.timer) {
                (true, Timer::Running(deadline)) => {
                    Timer::Paused(deadline.saturating_duration_since(now))
                }
                (false, Timer::Paused(remaining)) => Timer::Running(now + remaining),
                (_, timer) => timer,
            };
        }
    }

    pub fn history(&self) -> &[Notification] {
        &self.history
    }

    pub fn display(&self) -> Display {
        let mut items: Vec<Notification> = if self.expanded {
            self.history.clone()
        } else {
            self.previews
                .iter()
                .map(|preview| preview.notification.clone())
                .collect()
        };

        if !self.expanded {
            // Toasts render this order in reverse; critical alerts appear first.
            items.sort_by_key(|item| item.urgency);
        }

        Display {
            hidden: self.history.len().saturating_sub(items.len()),
            items,
            expanded: self.expanded,
        }
    }

    pub fn expire(&mut self) -> bool {
        self.expire_at(Instant::now())
    }

    fn expire_at(&mut self, now: Instant) -> bool {
        let previous_len = self.previews.len();

        self.previews.retain(|preview| match preview.timer {
            Timer::Running(deadline) => deadline > now,
            Timer::Never | Timer::Paused(_) => true,
        });

        previous_len != self.previews.len() && !self.expanded
    }

    pub fn next_wait(&self) -> Option<Duration> {
        self.next_wait_at(Instant::now())
    }

    fn next_wait_at(&self, now: Instant) -> Option<Duration> {
        self.previews
            .iter()
            .filter_map(|preview| match preview.timer {
                Timer::Running(deadline) => Some(deadline),
                Timer::Never | Timer::Paused(_) => None,
            })
            .min()
            .map(|deadline| deadline.saturating_duration_since(now))
    }
}

struct Preview {
    notification: Notification,
    timer: Timer,
}

#[derive(Clone, Copy)]
enum Timer {
    Never,
    Running(Instant),
    Paused(Duration),
}

fn expiration_timer(urgency: Urgency, timeout: i32, now: Instant, paused: bool) -> Timer {
    if urgency == Urgency::Critical {
        return Timer::Never;
    }

    let duration = match timeout {
        0 => return Timer::Never,
        value if value > 0 => Duration::from_millis(value as u64),
        _ => Duration::from_secs(if urgency == Urgency::Low { 3 } else { 5 }),
    };

    if paused {
        Timer::Paused(duration)
    } else {
        Timer::Running(now + duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification(id: u32, summary: &str) -> Notification {
        Notification {
            id,
            app: String::new(),
            icon: String::new(),
            desktop_entry: None,
            summary: summary.to_string(),
            body: String::new(),
            default_action: false,
            resident: false,
            urgency: Urgency::Normal,
            request_attention: false,
        }
    }

    #[test]
    fn previews_can_expand_into_full_history() {
        let mut center = Center::default();

        for id in 1..=5 {
            center.apply(Event::Added(notification(id, &id.to_string()), 0));
        }

        let collapsed = center.display();

        assert_eq!(
            collapsed
                .items
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            [3, 4, 5]
        );
        assert_eq!(collapsed.hidden, 2);

        center.expand(true);

        let expanded = center.display();

        assert_eq!(expanded.items.len(), 5);
        assert_eq!(expanded.hidden, 0);
    }

    #[test]
    fn urgency_controls_default_expiration_but_preserves_explicit_timeouts() {
        let start = Instant::now();

        for (urgency, timeout, expected) in [
            (Urgency::Low, -1, Some(Duration::from_secs(3))),
            (Urgency::Normal, -1, Some(Duration::from_secs(5))),
            (Urgency::Low, 120, Some(Duration::from_millis(120))),
            (Urgency::Normal, 0, None),
            (Urgency::Critical, 1, None),
        ] {
            let mut center = Center::default();
            let mut item = notification(1, "one");
            item.urgency = urgency;
            center.apply_at(Event::Added(item, timeout), start);
            assert_eq!(center.next_wait_at(start), expected);
            center.expire_at(start + Duration::from_secs(10));
            assert_eq!(
                center.display().items.len(),
                usize::from(expected.is_none())
            );
            assert_eq!(center.history().len(), 1);
        }
    }

    #[test]
    fn critical_alerts_have_preview_priority_and_remain_in_history() {
        let mut center = Center::default();
        let mut critical = notification(1, "critical");
        critical.urgency = Urgency::Critical;
        center.apply(Event::Added(critical, 1));

        for id in 2..=70 {
            center.apply(Event::Added(notification(id, "normal"), 0));
        }

        assert_eq!(center.history().len(), HISTORY_LIMIT);
        assert_eq!(center.history()[0].id, 1);
        assert_eq!(center.display().items.last().unwrap().id, 1);
        assert_eq!(center.display().items.len(), PREVIEW_LIMIT);
        center.hide_previews(&[1, 70]);
        assert!(!center.display().items.iter().any(|item| item.id == 1));
        assert_eq!(center.history()[0].id, 1);

        center.expand(true);
        assert_eq!(center.display().items[0].id, 1);
        center.expand(false);
        center.dismiss(1);
        assert!(!center.history().iter().any(|item| item.id == 1));
    }

    #[test]
    fn critical_alerts_use_the_preview_budget_but_remain_in_full_history() {
        let mut center = Center::default();

        for id in 1..=55 {
            let mut item = notification(id, "critical");
            item.urgency = Urgency::Critical;
            center.apply(Event::Added(item, -1));
        }

        assert_eq!(center.display().items.len(), PREVIEW_LIMIT);
        assert_eq!(center.display().hidden, 55 - PREVIEW_LIMIT);
        assert_eq!(
            center
                .display()
                .items
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            [53, 54, 55]
        );
        assert_eq!(center.history().len(), 55);
        assert_eq!(center.next_wait(), None);
        center.expand(true);
        assert_eq!(center.display().items.len(), 55);
        center.apply(Event::Closed(55));
        assert_eq!(center.display().items.len(), 54);
        center.expand(false);
        assert_eq!(center.display().items.len(), PREVIEW_LIMIT - 1);
    }

    #[test]
    fn hover_pause_preserves_remaining_preview_time() {
        let start = Instant::now();
        let mut center = Center::default();
        center.apply_at(Event::Added(notification(1, "one"), 100), start);
        center.set_hovered_at(true, start + Duration::from_millis(40));

        assert_eq!(center.next_wait_at(start + Duration::from_millis(40)), None);
        assert!(!center.expire_at(start + Duration::from_millis(500)));

        center.set_hovered_at(false, start + Duration::from_millis(500));

        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(500)),
            Some(Duration::from_millis(60))
        );
        assert!(!center.expire_at(start + Duration::from_millis(559)));
        assert!(center.expire_at(start + Duration::from_millis(560)));
    }

    #[test]
    fn popup_pause_preserves_existing_and_incoming_preview_time() {
        let start = Instant::now();
        let mut center = Center::default();
        center.apply_at(Event::Added(notification(1, "existing"), 100), start);
        center.set_popup_open_at(true, start + Duration::from_millis(40));
        center.apply_at(
            Event::Added(notification(2, "during popup"), 80),
            start + Duration::from_millis(50),
        );

        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(500)),
            None
        );
        assert!(!center.expire_at(start + Duration::from_millis(500)));
        assert_eq!(center.display().items.len(), 2);

        center.set_popup_open_at(false, start + Duration::from_millis(500));

        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(500)),
            Some(Duration::from_millis(60))
        );
        assert!(!center.expire_at(start + Duration::from_millis(559)));
        assert!(center.expire_at(start + Duration::from_millis(560)));
        assert_eq!(center.display().items[0].id, 2);
        assert!(center.expire_at(start + Duration::from_millis(580)));
        assert_eq!(center.history().len(), 2);
    }

    #[test]
    fn popup_clears_stale_hover_without_resuming_expanded_history() {
        let start = Instant::now();
        let mut center = Center::default();
        center.apply_at(Event::Added(notification(1, "existing"), 100), start);
        center.set_hovered_at(true, start + Duration::from_millis(40));
        center.set_popup_open_at(true, start + Duration::from_millis(50));
        // Queued pointer events from the hidden window must not keep it paused.
        center.set_hovered_at(true, start + Duration::from_millis(60));
        center.set_popup_open_at(false, start + Duration::from_millis(500));

        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(500)),
            Some(Duration::from_millis(60))
        );

        center.expand_at(true, start + Duration::from_millis(510));
        center.set_popup_open_at(true, start + Duration::from_millis(520));
        center.set_popup_open_at(false, start + Duration::from_millis(600));
        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(600)),
            None
        );

        center.expand_at(false, start + Duration::from_millis(700));
        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(700)),
            Some(Duration::from_millis(50))
        );
    }

    #[test]
    fn hidden_previews_remain_in_history_without_reappearing() {
        let mut center = Center::default();

        for id in 1..=5 {
            center.apply(Event::Added(notification(id, &id.to_string()), 0));
        }

        center.hide_previews(&[3, 4]);

        let display = center.display();

        assert_eq!(
            display.items.iter().map(|item| item.id).collect::<Vec<_>>(),
            [5]
        );
        assert_eq!(display.hidden, 4);
        assert_eq!(center.history().len(), 5);

        center.hide_previews(&[5]);

        assert!(center.display().items.is_empty());

        center.expand(true);

        assert_eq!(center.display().items.len(), 5);

        center.expand(false);

        assert!(center.display().items.is_empty());

        center.apply(Event::Added(notification(3, "updated"), 0));

        assert!(center.display().items.is_empty());
        assert_eq!(center.history().len(), 5);
        assert_eq!(center.history().last().unwrap().summary, "updated");

        let mut critical = notification(3, "escalated");
        critical.urgency = Urgency::Critical;
        center.apply(Event::Added(critical, 0));
        assert_eq!(center.display().items[0].id, 3);
        assert_eq!(center.history().len(), 5);
    }

    #[test]
    fn replacing_an_expired_preview_stays_quiet_until_urgency_increases() {
        let mut center = Center::default();
        let start = Instant::now();

        center.apply_at(Event::Added(notification(1, "Low battery"), -1), start);
        assert!(center.expire_at(start + Duration::from_secs(5)));
        center.apply_at(
            Event::Added(notification(1, "14% remaining"), -1),
            start + Duration::from_secs(6),
        );
        assert!(center.display().items.is_empty());
        assert_eq!(center.history().len(), 1);
        assert_eq!(center.history()[0].summary, "14% remaining");

        let mut critical = notification(1, "Critical battery");
        critical.urgency = Urgency::Critical;
        center.apply_at(Event::Added(critical, -1), start + Duration::from_secs(7));
        assert_eq!(center.display().items.len(), 1);
        assert_eq!(center.next_wait(), None);
    }

    #[test]
    fn explicit_attention_resurfaces_an_expired_preview_without_a_duplicate_history_entry() {
        let mut center = Center::default();
        let start = Instant::now();

        center.apply_at(Event::Added(notification(1, "warning"), -1), start);
        assert!(center.expire_at(start + Duration::from_secs(5)));

        let mut renewed = notification(1, "renewed warning");
        renewed.request_attention = true;
        center.apply_at(Event::Added(renewed, -1), start + Duration::from_secs(6));
        assert_eq!(center.display().items[0].id, 1);
        assert_eq!(center.history().len(), 1);
        assert_eq!(
            center.next_wait_at(start + Duration::from_secs(6)),
            Some(Duration::from_secs(5))
        );
    }

    #[test]
    fn expanded_pause_handles_new_and_repeated_notifications() {
        let start = Instant::now();
        let mut center = Center::default();
        center.apply_at(Event::Added(notification(1, "old"), 100), start);
        center.expand_at(true, start + Duration::from_millis(40));
        center.apply_at(
            Event::Added(notification(1, "updated"), 200),
            start + Duration::from_millis(50),
        );

        center.apply_at(
            Event::Added(notification(2, "during pause"), 80),
            start + Duration::from_millis(60),
        );

        assert_eq!(center.history()[0].summary, "updated");
        assert!(!center.expire_at(start + Duration::from_millis(90)));

        center.set_hovered_at(true, start + Duration::from_millis(90));
        center.expand_at(false, start + Duration::from_millis(100));
        center.set_hovered_at(false, start + Duration::from_millis(110));

        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(110)),
            Some(Duration::from_millis(80))
        );
        assert!(center.expire_at(start + Duration::from_millis(190)));
        assert_eq!(center.history().len(), 2);
        assert_eq!(
            center.next_wait_at(start + Duration::from_millis(190)),
            Some(Duration::from_millis(120))
        );
    }
}
