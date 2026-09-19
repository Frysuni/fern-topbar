use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct Connection {
    pub initial: HashSet<String>,
    pub events: UnboundedReceiver<HashSet<String>>,
}

pub fn connect() -> Result<Connection, String> {
    let (initial, events) = wayland::watch()?;

    Ok(Connection { initial, events })
}

mod wayland {
    use std::collections::{HashMap, HashSet};
    use std::thread;

    use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
    use wayland_client::backend::ObjectId;
    use wayland_client::protocol::{wl_output, wl_registry};
    use wayland_client::{
        Connection as WaylandConnection, Dispatch, EventQueue, Proxy, QueueHandle,
        event_created_child,
    };
    use wayland_protocols_wlr::foreign_toplevel::v1::client::{
        zwlr_foreign_toplevel_handle_v1 as handle, zwlr_foreign_toplevel_manager_v1 as manager,
    };

    pub fn watch() -> Result<(HashSet<String>, UnboundedReceiver<HashSet<String>>), String> {
        let mut runtime = WatcherRuntime::connect()?;
        let initial = runtime.tracker.focused_fullscreen_outputs();
        let (events, receiver) = mpsc::unbounded_channel();

        runtime.tracker.publisher.attach(events, initial.clone());
        runtime.start();

        Ok((initial, receiver))
    }

    struct WatcherRuntime {
        queue: EventQueue<Tracker>,
        tracker: Tracker,
    }

    impl WatcherRuntime {
        fn connect() -> Result<Self, String> {
            ProtocolDiscovery::connect()?.bind()
        }

        fn start(mut self) {
            thread::spawn(
                move || {
                    while self.queue.blocking_dispatch(&mut self.tracker).is_ok() {}
                },
            );
        }
    }

    struct ProtocolDiscovery {
        queue: EventQueue<Tracker>,
        handle: QueueHandle<Tracker>,
        registry: wl_registry::WlRegistry,
        tracker: Tracker,
    }

    impl ProtocolDiscovery {
        fn connect() -> Result<Self, String> {
            let connection = WaylandConnection::connect_to_env().map_err(|error| {
                format!("cannot connect to Wayland for fullscreen state: {error}")
            })?;
            let mut queue = connection.new_event_queue();
            let handle = queue.handle();
            let registry = connection.display().get_registry(&handle, ());
            let mut tracker = Tracker::default();

            queue
                .roundtrip(&mut tracker)
                .map_err(|error| format!("cannot list Wayland protocols: {error}"))?;

            Ok(Self {
                queue,
                handle,
                registry,
                tracker,
            })
        }

        fn bind(mut self) -> Result<WatcherRuntime, String> {
            let manager_global = self
                .tracker
                .global("zwlr_foreign_toplevel_manager_v1", 2)
                .ok_or_else(|| "compositor does not expose fullscreen window state".to_string())?;

            // EventQueue owns the connection and registered object data. Proxy
            // handles need not be retained: dropping one sends no destroy request.
            self.registry
                .bind::<manager::ZwlrForeignToplevelManagerV1, _, _>(
                    manager_global.name,
                    manager_global.version.min(3),
                    &self.handle,
                    (),
                );

            if self.bind_named_outputs() == 0 {
                return Err("compositor does not expose named Wayland outputs".into());
            }

            self.queue
                .roundtrip(&mut self.tracker)
                .map_err(|error| format!("cannot read fullscreen window state: {error}"))?;

            Ok(WatcherRuntime {
                queue: self.queue,
                tracker: self.tracker,
            })
        }

        fn bind_named_outputs(&mut self) -> usize {
            let mut count = 0;

            for global in &self.tracker.globals {
                if !global.matches("wl_output", 4) {
                    continue;
                }

                let output = self.registry.bind::<wl_output::WlOutput, _, _>(
                    global.name,
                    global.version.min(4),
                    &self.handle,
                    (),
                );

                self.tracker.output_ids.insert(global.name, output.id());
                count += 1;
            }

            count
        }
    }

    #[derive(Clone)]
    struct Global {
        name: u32,
        interface: String,
        version: u32,
    }

    impl Global {
        fn matches(&self, interface: &str, minimum_version: u32) -> bool {
            self.interface == interface && self.version >= minimum_version
        }
    }

    #[derive(Default)]
    struct Tracker {
        globals: Vec<Global>,
        output_ids: HashMap<u32, ObjectId>,
        output_names: HashMap<ObjectId, String>,
        toplevels: HashMap<ObjectId, Toplevel>,
        publisher: Publisher,
    }

    impl Tracker {
        fn global(&self, interface: &str, minimum_version: u32) -> Option<Global> {
            self.globals
                .iter()
                .find(|global| global.matches(interface, minimum_version))
                .cloned()
        }

        fn focused_fullscreen_outputs(&self) -> HashSet<String> {
            self.toplevels
                .values()
                .filter(|window| window.fullscreen && window.focused)
                .flat_map(|window| &window.outputs)
                .filter_map(|id| self.output_names.get(id).cloned())
                .collect()
        }

        fn publish(&mut self) {
            let outputs = self.focused_fullscreen_outputs();

            self.publisher.publish(outputs);
        }
    }

    #[derive(Default)]
    struct Publisher {
        last: Option<HashSet<String>>,
        sender: Option<UnboundedSender<HashSet<String>>>,
    }

    impl Publisher {
        fn attach(&mut self, sender: UnboundedSender<HashSet<String>>, current: HashSet<String>) {
            self.last = Some(current);
            self.sender = Some(sender);
        }

        fn publish(&mut self, outputs: HashSet<String>) {
            if self.last.as_ref() == Some(&outputs) {
                return;
            }

            self.last = Some(outputs.clone());

            if let Some(sender) = &self.sender {
                let _ = sender.send(outputs);
            }
        }
    }

    #[derive(Default)]
    struct Toplevel {
        fullscreen: bool,
        focused: bool,
        outputs: HashSet<ObjectId>,
    }

    impl Toplevel {
        fn update_state(&mut self, state: &[u8]) {
            self.fullscreen = false;
            self.focused = false;

            for value in state.as_chunks::<4>().0 {
                match u32::from_ne_bytes(*value) {
                    value if value == handle::State::Fullscreen as u32 => {
                        self.fullscreen = true;
                    }
                    value if value == handle::State::Activated as u32 => {
                        self.focused = true;
                    }
                    _ => {}
                }
            }
        }
    }

    impl Dispatch<wl_registry::WlRegistry, ()> for Tracker {
        fn event(
            tracker: &mut Self,
            _: &wl_registry::WlRegistry,
            event: wl_registry::Event,
            _: &(),
            _: &WaylandConnection,
            _: &QueueHandle<Self>,
        ) {
            match event {
                wl_registry::Event::Global {
                    name,
                    interface,
                    version,
                } => {
                    tracker.globals.push(Global {
                        name,
                        interface,
                        version,
                    });
                }
                wl_registry::Event::GlobalRemove { name } => {
                    if let Some(id) = tracker.output_ids.remove(&name) {
                        tracker.output_names.remove(&id);
                        tracker.publish();
                    }
                }
                _ => {}
            }
        }
    }

    impl Dispatch<wl_output::WlOutput, ()> for Tracker {
        fn event(
            tracker: &mut Self,
            output: &wl_output::WlOutput,
            event: wl_output::Event,
            _: &(),
            _: &WaylandConnection,
            _: &QueueHandle<Self>,
        ) {
            if let wl_output::Event::Name { name } = event {
                tracker.output_names.insert(output.id(), name);
                tracker.publish();
            }
        }
    }

    impl Dispatch<manager::ZwlrForeignToplevelManagerV1, ()> for Tracker {
        fn event(
            tracker: &mut Self,
            _: &manager::ZwlrForeignToplevelManagerV1,
            event: manager::Event,
            _: &(),
            _: &WaylandConnection,
            _: &QueueHandle<Self>,
        ) {
            match event {
                manager::Event::Toplevel { toplevel } => {
                    tracker.toplevels.insert(toplevel.id(), Toplevel::default());
                }
                manager::Event::Finished => {
                    tracker.toplevels.clear();
                    tracker.publish();
                }
                _ => {}
            }
        }

        event_created_child!(Tracker, manager::ZwlrForeignToplevelManagerV1, [
            0 => (handle::ZwlrForeignToplevelHandleV1, ()),
        ]);
    }

    impl Dispatch<handle::ZwlrForeignToplevelHandleV1, ()> for Tracker {
        fn event(
            tracker: &mut Self,
            window: &handle::ZwlrForeignToplevelHandleV1,
            event: handle::Event,
            _: &(),
            _: &WaylandConnection,
            _: &QueueHandle<Self>,
        ) {
            let id = window.id();

            match event {
                handle::Event::State { state } => {
                    if let Some(window) = tracker.toplevels.get_mut(&id) {
                        window.update_state(&state);
                    }
                }
                handle::Event::OutputEnter { output } => {
                    if let Some(window) = tracker.toplevels.get_mut(&id) {
                        window.outputs.insert(output.id());
                    }
                }
                handle::Event::OutputLeave { output } => {
                    if let Some(window) = tracker.toplevels.get_mut(&id) {
                        window.outputs.remove(&output.id());
                    }
                }
                handle::Event::Done => tracker.publish(),
                handle::Event::Closed => {
                    tracker.toplevels.remove(&id);
                    tracker.publish();
                }
                _ => {}
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn fullscreen_window_blocks_hover_only_while_focused() {
            let id = ObjectId::null();
            let mut tracker = Tracker::default();

            tracker.output_names.insert(id.clone(), "HDMI-A-1".into());
            tracker.toplevels.insert(
                id.clone(),
                Toplevel {
                    outputs: HashSet::from([id.clone()]),
                    ..Toplevel::default()
                },
            );

            let fullscreen = (handle::State::Fullscreen as u32).to_ne_bytes();
            let activated = (handle::State::Activated as u32).to_ne_bytes();

            tracker
                .toplevels
                .get_mut(&id)
                .unwrap()
                .update_state(&fullscreen);

            assert!(tracker.focused_fullscreen_outputs().is_empty());

            tracker
                .toplevels
                .get_mut(&id)
                .unwrap()
                .update_state(&[fullscreen, activated].concat());

            assert_eq!(
                tracker.focused_fullscreen_outputs(),
                HashSet::from(["HDMI-A-1".to_owned()])
            );

            tracker
                .toplevels
                .get_mut(&id)
                .unwrap()
                .update_state(&fullscreen);

            assert!(tracker.focused_fullscreen_outputs().is_empty());
        }
    }
}
