use std::{error::Error, future::pending};
use std::process::Command;
use zbus::{connection, interface, proxy};
use int_enum::IntEnum;

// References:
// https://github.com/xremap/xremap/blob/master/src/client/kde_client.rs
// https://docs.rs/zbus/latest/zbus/

struct ActiveWindow {
    res_class: String,
    res_name: String,
    title: String,
}

struct GesturesHelper {
    active_window: ActiveWindow,
}


#[proxy(
interface = "org.kde.kglobalaccel.Component",
default_service = "org.kde.kglobalaccel",
default_path = "/component/kwin",
)]
trait KWinGlobalAccel {
    #[zbus(name = "invokeShortcut")]
    async fn invoke_shortcut(&self, shortcut: &str) -> zbus::Result<()>;
}

#[proxy(
interface = "org.kde.KWin",
default_service = "org.kde.KWin",
default_path = "/KWin",
)]
trait KWin {
    #[zbus(name = "previousDesktop")]
    async fn previousDesktop(&self) -> zbus::Result<()>;
    #[zbus(name = "nextDesktop")]
    async fn nextDesktop(&self) -> zbus::Result<()>;
}

#[repr(u8)]
#[derive(IntEnum, Debug, PartialEq)]
enum Gesture {
    ThreeSwipeUp = 0,
    ThreeSwipeDown = 1,
    ThreeSwipeLeft = 2,
    ThreeSwipeRight = 3,
    FourSwipeUp = 4,
    FourSwipeDown = 5,
    FourSwipeLeft = 6,
    FourSwipeRight = 7,
    TwoPinchIn = 8,
    TwoPinchOut = 9,
    ThreePinchIn = 10,
    ThreePinchOut = 11,
    FourPinchIn = 12,
    FourPinchOut = 13,
}

#[proxy(
default_service = "org.freedesktop.DBus",
default_path = "/org/freedesktop/DBus",
)]
trait DBus {
    #[zbus(name = "ListNames")]
    async fn list_names(&self) -> zbus::Result<Vec<String>>;
}

fn ydotool_key(key: &str) {
    Command::new("ydotool")
        .args(&["key", key])
        .spawn()
        .expect("Failed to execute command");
}

async fn call_dolphin_action(action: &str) {
    let conn = connection::Connection::session().await.unwrap();
    let names = DBusProxy::new(&conn).await.unwrap().list_names().await.unwrap();
    for name in names {
        if name.starts_with("org.kde.dolphin") {
            let is_active_window = conn.call_method(
                Some(name.clone()),
                "/dolphin/Dolphin_1",
                Some("org.kde.dolphin.MainWindow"),
                "isActiveWindow",
                &(),
            ).await.unwrap().body().deserialize::<bool>().unwrap();

            if is_active_window {
                conn.call_method(
                    Some(name.clone()),
                    "/dolphin/Dolphin_1",
                    Some("org.kde.KMainWindow"),
                    "activateAction",
                    &action,
                ).await.unwrap();
                break;
            }
        }
    }
}

#[interface(name = "ink.chyk.GesturesHelper")]
impl GesturesHelper {
    fn notify_active_window(
        &mut self, title: String, res_class: String, res_name: String,
    ) {
        self.active_window.title = title;
        self.active_window.res_class = res_class;
        self.active_window.res_name = res_name;
    }
    fn get_active_window(&self) -> (String, String, String) {
        (
            self.active_window.title.clone(),
            self.active_window.res_class.clone(),
            self.active_window.res_name.clone(),
        )
    }
    async fn invoke_gesture(&self, gesture_id: u8) {
        let gesture = Gesture::try_from(gesture_id).unwrap();
        let conn = connection::Connection::session().await.unwrap();
        match gesture {
            Gesture::ThreeSwipeUp | Gesture::ThreeSwipeDown => {
                let kwin_global_accel = KWinGlobalAccelProxy::new(&conn).await.unwrap();
                kwin_global_accel.invoke_shortcut(match &gesture {
                    Gesture::ThreeSwipeUp => "Cycle Overview Opposite",
                    Gesture::ThreeSwipeDown => "Cycle Overview",
                    _ => "",
                }).await.unwrap();
            },
            Gesture::ThreeSwipeRight => {
                match self.active_window.res_class.as_str() {
                    "org.telegram.desktop" => {
                        ydotool_key("1:1");  // 按下 Esc
                        ydotool_key("1:0");  // 松开 Esc

                    },
                    "org.kde.dolphin" => call_dolphin_action("go_back").await,
                    _ => {
                        let kwin = KWinProxy::new(&conn).await.unwrap();
                        kwin.previousDesktop().await.unwrap();
                    }
                }
            },
            Gesture::FourSwipeRight => {
                let kwin = KWinProxy::new(&conn).await.unwrap();
                kwin.previousDesktop().await.unwrap();
            },
            Gesture::ThreeSwipeLeft => {
                match self.active_window.res_class.as_str() {
                    "org.kde.dolphin" => call_dolphin_action("go_forward").await,
                    _ => {
                        let kwin = KWinProxy::new(&conn).await.unwrap();
                        kwin.nextDesktop().await.unwrap();
                    }
                }
            },
            Gesture::FourSwipeLeft => {
                let kwin = KWinProxy::new(&conn).await.unwrap();
                kwin.nextDesktop().await.unwrap();
            },
            Gesture::ThreePinchIn => {
                let kwin_global_accel = KWinGlobalAccelProxy::new(&conn).await.unwrap();
                kwin_global_accel.invoke_shortcut("view_zoom_out").await.unwrap();
            },
            Gesture::ThreePinchOut => {
                let kwin_global_accel = KWinGlobalAccelProxy::new(&conn).await.unwrap();
                kwin_global_accel.invoke_shortcut("view_zoom_in").await.unwrap();
            }
            _ => {}
        }
    }
}


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let gestures_helper = GesturesHelper {
        active_window: ActiveWindow {
            res_class: String::new(),
            res_name: String::new(),
            title: String::new(),
        }
    };
    let _conn = connection::Builder::session()?
        .name("ink.chyk.GesturesHelper")?
        .serve_at("/ink/chyk/GesturesHelper", gestures_helper)?
        .build()
        .await?;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
