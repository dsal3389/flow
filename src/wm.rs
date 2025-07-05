use std::sync::Arc;
use std::collections::HashSet;

use tokio::sync::Mutex;
use tokio::task::JoinSet;

use x11rb_async::errors::ReplyError;
use x11rb_async::connection::Connection;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{
    ConnectionExt as _, ChangeWindowAttributesAux, ConfigureWindowAux, EventMask, GrabMode,
    KeyPressEvent, KeyReleaseEvent, MapRequestEvent, ConfigureRequestEvent, ModMask, Window,
};
use x11rb_async::protocol::{ErrorKind, Event};
use xkbcommon::xkb;

use crate::Config;
use crate::key::{Key, KeyState};

use crate::combos::{ComboTree, ComboRecord};
use crate::combos::handlers::Spawn;

pub struct WindowManager<C>
where
    C: Connection + Sync + Send + 'static,
{
    config: Config,
    connection: C,
    root: Window,
    keystate: KeyState,
    combos_tree: Mutex<ComboTree>,
    combos_record: Mutex<ComboRecord>,
}

impl<C> WindowManager<C>
where
    C: Connection + Sync + Send + 'static,
{
    /// creates a new window manager from the given connection
    /// will try to change the root window attributes to register
    /// as window manager
    pub async fn with_connection(
        connection: C,
        root: Window,
        config: Config,
    ) -> anyhow::Result<Self> {
        connection
            .change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new()
                    .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT),
            )
            .await?
            .check()
            .await
            .inspect_err(|err| {
                if let ReplyError::X11Error(err) = err {
                    if err.error_kind == ErrorKind::Access {
                        log::error!(
                            "window manager already runnig, couldn't request event from x11 server"
                        );
                    }
                }
            })?;

        connection.xkb_use_extension(1, 0).await?;
        let keystate = KeyState::from_connection(&connection).await?;

        Ok(WindowManager {
            config,
            connection,
            root,
            keystate,
            combos_tree: Mutex::new(ComboTree::default()),
            combos_record: Mutex::new(ComboRecord::default()),
        })
    }

    /// running the window manager will register for keybinds defined in the
    /// configuration and listen/handle events from X11
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        self.clone().setup_binds().await?;

        loop {
            match self.connection.wait_for_event().await? {
                Event::KeyPress(event) => self.handle_key_press_event(event).await,
                Event::KeyRelease(event) => self.handle_key_release_event(event).await,
                Event::MapRequest(event) => self.handle_map_request_event(event).await?,
                Event::ConfigureRequest(event) => {
                    self.connection
                        .configure_window(
                            event.window,
                            &ConfigureWindowAux {
                                x: Some(event.x as i32),
                                y: Some(event.y as i32),
                                width: Some(event.width as u32),
                                height: Some(event.height as u32),
                                border_width: None,
                                sibling: None,
                                stack_mode: None,
                            },
                        )
                        .await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// this method is temporarly here for now, in future need to check
    /// when the config file is edited and refresh the binds
    async fn setup_binds(self: Arc<Self>) -> anyhow::Result<()> {
        // here for future, when `setup_binds` will be
        // called multiple times
        self.combos_tree.lock().await.clear();
        self.connection
            .ungrab_key(0, self.root, ModMask::ANY)
            .await?
            .check()
            .await?;

        let mut root_keycodes = HashSet::new();

        // iterator on the config binds, for each bind we register the
        // combo and add the chars to the `keycodes_to_register` set
        // so we will later request those key press events from the X server
        for (name, config_combo) in self.config.combos() {
            let keycode_combo: Vec<xkb::Keycode> = config_combo
                .keys()
                .iter()
                .filter_map(|key| {
                    key.chars().next().and_then(|c| {
                        let key = Key::from(c);
                        key.keycode(&self.keystate)
                    })
                })
                .collect();

            self.combos_tree.lock().await.add_combo(
                &keycode_combo,
                Arc::new(Spawn::new(
                    name.clone(),
                    "alacritty".to_string(),
                    Vec::new(),
                )),
            );

            // we only need to register the first key in the combo
            // and x11 will report all keypresses while the root
            // key is pressed first
            root_keycodes.insert(keycode_combo.first().take().unwrap().clone());
        }

        let mut tasks = JoinSet::<anyhow::Result<()>>::new();

        // create an async task for each key that is needed to be grabbed
        for keycode in root_keycodes {
            let wm = Arc::clone(&self);
            tasks.spawn(async move {
                wm.connection
                    .grab_key(
                        true,
                        wm.root,
                        wm.config.flow().modifier().into(),
                        keycode,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    )
                    .await?
                    .check()
                    .await?;
                Ok(())
            });
        }

        // wait for all grab keys to finish
        // before we continue, we don't care for now if some failed
        // or suceed
        let _ = tasks.join_all().await;
        Ok(())
    }

    #[inline]
    async fn handle_key_press_event(&self, event: KeyPressEvent) {
        let combo_snapshot = {
            let mut combo_record = self.combos_record.lock().await;
            combo_record.add(event.detail.into());
            combo_record.snapshot()
        };

        if let Some(handler) = self
            .combos_tree
            .lock()
            .await
            .find_combo_handler(combo_snapshot)
        {
            log::info!("handler found {}", handler.handler_name());
            let _ = handler.handle().await.inspect_err(|err| {
                log::error!(
                    "handler `{}` returned an error while trying to execute, {}",
                    handler.handler_name(),
                    err
                );
            });
        }
    }

    #[inline]
    async fn handle_key_release_event(&self, event: KeyReleaseEvent) {
        self.combos_record.lock().await.remove(event.detail.into());
    }

    #[inline]
    async fn handle_map_request_event(&self, event: MapRequestEvent) -> anyhow::Result<()> {
        let attributes = self
            .connection
            .get_window_attributes(event.window)
            .await?
            .reply()
            .await?;

        if attributes.override_redirect {
            return Ok(());
        }

        self.connection.map_window(event.window).await?;
        self.connection
            .configure_window(
                event.window,
                &ConfigureWindowAux {
                    x: Some(0),
                    y: Some(0),
                    width: Some(100),
                    height: Some(100),
                    border_width: Some(4),
                    sibling: None,
                    stack_mode: None,
                },
            )
            .await?;
        Ok(())
    }
}
