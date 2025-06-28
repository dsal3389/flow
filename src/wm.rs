use std::sync::Arc;
use std::collections::HashSet;

use tokio::sync::Mutex;
use tokio::task::JoinSet;

use x11rb_async::errors::ReplyError;
use x11rb_async::connection::Connection;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{
    ConnectionExt as _, ChangeWindowAttributesAux, EventMask, GrabMode, ModMask, Window,
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
{config: Config,
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
    pub async fn from_connection(
        connection: C,
        root: Window,
        config: Config,
    ) -> anyhow::Result<Self> {
        let attributes = ChangeWindowAttributesAux::new()
            .event_mask(EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT);
        connection
            .change_window_attributes(root, &attributes)
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
                Event::KeyPress(event) => {
                    let combo_snapshot = {
                        let mut combo_record = self.combos_record.lock().await;
                        combo_record.add(event.detail.into());
                        combo_record.snapshot()
                    };

                    log::info!("combo {}", combo_snapshot);
                    let handler = self.combos_tree.lock().await.find_combo_handler(combo_snapshot);
                    log::info!("found handler {}", handler.is_some());
                }
                Event::KeyRelease(event) => {
                    self.combos_record.lock().await.remove(event.detail.into());
                }
                Event::MapRequest(event) => {
                    log::info!("got window map request");
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn setup_binds(self: Arc<Self>) -> anyhow::Result<()> {
        let mut root_keycodes = HashSet::new();

        // iterator on the config binds, for each bind we register the
        // combo and add the chars to the `keycodes_to_register` set
        // so we will later request those key press events from the X server
        for (name, config_bind) in self.config.binds() {
            let keycode_combo: Vec<xkb::Keycode> = config_bind
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
                Arc::new(Spawn::new(name.clone(), "alacritty".to_string(), Vec::new())),
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
                        ModMask::CONTROL,
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
}
