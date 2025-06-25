use std::sync::Arc;
use std::collections::HashSet;

use tokio::sync::Mutex;
use tokio::task::JoinSet;

use x11rb_async::errors::ReplyError;
use x11rb_async::connection::Connection;
use x11rb_async::protocol::xkb::ConnectionExt as _;
use x11rb_async::protocol::xproto::{
    ConnectionExt as _,
    ChangeWindowAttributesAux,
    EventMask,
    GrabMode,
    ModMask,
    Window
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
    keystate: KeyState,
    connection: C,
    root: Window,
    combos: Mutex<ComboTree>,
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
            combos: Mutex::new(ComboTree::default()),
            combos_record: Mutex::new(ComboRecord::default())
        })
    }

    /// running the window manager will register for keybinds defined in the
    /// configuration and listen/handle events from X11
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        self.clone().setup_binds().await?;

        loop {
            match self.connection.wait_for_event().await? {
                Event::KeyPress(event) => {
                    let combo = {
                        let mut combo_record = self.combos_record.lock().await;
                        combo_record.press(event.detail);
                        combo_record.combo().to_vec()
                    };
                    self.combos_record.lock().await.press(event.detail);

                    let binds = self.combos.lock().await;
                    let found_combo = binds.find_combo_handler(&combo);
                    log::info!("key press, found combo {}, record {:?}", found_combo.is_some(), self.combos_record.lock().await);
                }
                Event::KeyRelease(event) => {
                    self.combos_record.lock().await.release(event.detail);
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn setup_binds(self: Arc<Self>) -> anyhow::Result<()> {
        self.connection.ungrab_key(
            0,
            self.root,
            ModMask::ANY
        ).await?.check().await?;

        let mut keycodes_to_register = HashSet::new();

        // iterator on the config binds, for each bind we register the
        // combo and add the chars to the `keycodes_to_register` set
        // so we will later request those key press events from the X server
        for (_, config_bind) in self.config.binds() {
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

            self.combos
                .lock()
                .await
                .add_combo(&keycode_combo, Box::new(Spawn::default()));
            keycodes_to_register.extend(keycode_combo);
        }

        let mut tasks = JoinSet::<anyhow::Result<()>>::new();

        // create an async task for each key that is needed to be grabbed
        for keycode in keycodes_to_register {
            let wm = Arc::clone(&self);
            tasks.spawn(async move {
                wm.connection
                    .grab_key(
                        true,
                        wm.root,
                        ModMask::ANY,
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
