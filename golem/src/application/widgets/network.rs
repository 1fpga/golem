use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Transform;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// Check that an address is valid (not 169.254.0.0/16) and is local.
fn is_valid(iface: &NetworkInterface) -> bool {
    // Check that there is at least one IP address.
    !iface.addr.is_empty()
        && iface.addr.iter().any(|addr| match addr {
            Addr::V4(addr) => {
                match addr.ip.octets() {
                    // 169.254.x.x
                    [169, 254, _, _] => false,
                    // Loopback
                    [127, _, _, _] => false,
                    _ => true,
                }
            }
            Addr::V6(_addr) => {
                // Impossible to know, but we filter those out for the moment...
                false
            }
        })
}

fn is_local(iface: &NetworkInterface) -> bool {
    iface.addr.iter().any(|addr| {
        match addr {
            Addr::V4(addr) => {
                match addr.ip.octets() {
                    // Class A
                    [10, _, _, _] => true,
                    // Class B
                    [172, x, _, _] if (16u8..=31).contains(&x) => true,
                    // Class C
                    [192, 168, _, _] => true,
                    _ => false,
                }
            }
            Addr::V6(_addr) => {
                // Impossible to know, but we filter those out for the moment...
                false
            }
        }
    })
}

pub fn is_wifi(iface: &NetworkInterface) -> bool {
    iface.name.starts_with("wlan")
}

#[derive(Debug, Default)]
struct NetworkStatus {
    pub local: bool,
    pub wifi: bool,
    pub internet: bool,
}

impl NetworkStatus {
    pub fn update(&mut self) -> bool {
        let mut changed = false;
        let connections = match network_interface::NetworkInterface::show() {
            Ok(connections) => connections,
            Err(_) => return false,
        };

        let mut ifaces = connections.into_iter().filter(is_valid);

        // Local is any connection that's not `wlan` and has an IP address that's not 169.254.x.x.
        let has_local = ifaces
            .clone()
            .any(|iface| is_local(&iface) && !is_wifi(&iface));
        if has_local != self.local {
            changed = true;
            self.local = has_local;
        }

        let has_wifi = ifaces.any(|iface| is_wifi(&iface));
        if has_wifi != self.wifi {
            changed = true;
            self.wifi = has_wifi;
        }

        // TODO: find a place to ping instead of opening a TCP connection to Google.
        let has_internet = std::net::TcpStream::connect("google.com:80").is_ok();
        if has_internet != self.internet {
            changed = true;
            self.internet = has_internet;
        }

        changed
    }
}

#[derive(Clone)]
pub struct NetworkWidgetView {
    position: Point,
    icons: [ImageRaw<'static, BinaryColor>; 3],
}

impl NetworkWidgetView {
    pub fn from_network(widget: &NetworkWidget) -> Self {
        let mut icons = [ImageRaw::new(&[], 0); 3];
        let mut i = 0;

        if widget.show_local {
            icons[i] = widget.icon_local;
            i += 1;
        }

        if widget.show_wifi {
            icons[i] = widget.icon_wifi;
            i += 1;
        }

        if widget.show_internet {
            icons[i] = widget.icon_internet;
        }

        Self {
            position: Point::zero(),
            icons,
        }
    }
}

impl Dimensions for NetworkWidgetView {
    fn bounding_box(&self) -> Rectangle {
        if self.icons.is_empty() {
            Rectangle::new(self.position, Size::zero())
        } else {
            let len = self.icons.len() as u32;
            Rectangle::new(self.position, Size::new(len * 8 + (len - 1) * 2, 8))
        }
    }
}

impl Transform for NetworkWidgetView {
    fn translate(&self, by: Point) -> Self {
        let mut new = self.clone();
        new.translate_mut(by);
        new
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.position += by;
        self
    }
}

impl Drawable for NetworkWidgetView {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut position = self.position;
        for icon in &self.icons {
            Image::new(icon, position).draw(target)?;
            position += Size::new(10, 0);
        }
        Ok(())
    }
}

/// A widget that shows the status of the network.
pub struct NetworkWidget {
    icon_local: ImageRaw<'static, BinaryColor>,
    icon_wifi: ImageRaw<'static, BinaryColor>,
    icon_internet: ImageRaw<'static, BinaryColor>,

    show_local: bool,
    show_wifi: bool,
    show_internet: bool,

    status: Arc<RwLock<NetworkStatus>>,
    dirty: Arc<AtomicBool>,

    quit_send: std::sync::mpsc::Sender<()>,
}

impl NetworkWidget {
    pub fn new() -> Self {
        let status = Arc::new(RwLock::new(NetworkStatus::default()));
        let dirty = Arc::new(AtomicBool::new(true));
        let (quit_send, quit_recv) = std::sync::mpsc::channel();

        std::thread::spawn({
            let status = status.clone();
            let dirty = dirty.clone();
            move || loop {
                loop {
                    if let Ok(mut status) = status.write() {
                        if status.update() {
                            dirty.store(true, Ordering::Relaxed);
                        }
                    }

                    if quit_recv
                        .recv_timeout(std::time::Duration::from_secs(5))
                        .is_ok()
                    {
                        break;
                    }
                }
            }
        });

        let icon_local = ImageRaw::new(include_bytes!("../../../assets/icons/network_eth.raw"), 8);
        let icon_wifi = ImageRaw::new(include_bytes!("../../../assets/icons/network_wifi.raw"), 8);
        let icon_internet =
            ImageRaw::new(include_bytes!("../../../assets/icons/network_globe.raw"), 8);

        Self {
            status,
            dirty,
            icon_local,
            icon_wifi,
            icon_internet,
            show_local: false,
            show_wifi: false,
            quit_send,
            show_internet: false,
        }
    }

    pub fn update(&mut self) -> bool {
        if self.dirty.load(Ordering::Relaxed) {
            if let Ok(status) = self.status.read() {
                self.show_local = status.local;
                self.show_wifi = status.wifi;
                self.show_internet = status.internet;

                self.dirty.store(false, Ordering::Relaxed);
                return true;
            }
        }
        false
    }
}

impl Drop for NetworkWidget {
    fn drop(&mut self) {
        self.quit_send.send(()).unwrap();
    }
}
