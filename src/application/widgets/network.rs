use crate::macguiver::buffer::DrawBuffer;
use crate::macguiver::widgets::group::HorizontalWidgetGroup;
use crate::macguiver::widgets::iconoir::IconoirWidget;
use crate::macguiver::widgets::Widget;
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Check that an address is valid (not 169.254.0.0/16) and is local.
pub fn is_valid(iface: &NetworkInterface) -> bool {
    iface.addr.iter().all(|addr| match addr {
        Addr::V4(addr) => {
            match addr.ip.octets() {
                // 169.254.x.x
                [169, 254, _, _] => false,
                // Loopback
                [127, _, _, _] => false,
                // Class A
                [10, _, _, _] => true,
                // Class B
                [172, x, _, _] if (16u8..=31).contains(&x) => true,
                // Class C
                [192, 168, _, _] => true,
                _ => false,
            }
        }
        Addr::V6(addr) => {
            // Impossible to know...
            return true;
        }
    })
}

#[derive(Debug, Default)]
struct NetworkStatus {
    pub local: AtomicBool,
    pub wifi: AtomicBool,
    pub internet: AtomicBool,
}

impl NetworkStatus {
    pub fn update(&self) {
        let connections = match network_interface::NetworkInterface::show() {
            Ok(connections) => connections,
            Err(_) => return,
        };

        let mut ifaces = connections.into_iter().filter(is_valid);

        // Local is any connection that's not `wlan` and has an IP address that's not 169.254.x.x.
        self.local.store(
            ifaces.clone().any(|iface| !iface.name.starts_with("wlan")),
            Ordering::Relaxed,
        );

        self.wifi.store(
            ifaces.any(|iface| iface.name.starts_with("wlan")),
            Ordering::Relaxed,
        );

        // TODO: find a place to ping instead.
        self.internet.store(
            std::net::TcpStream::connect("google.com:80").is_ok(),
            Ordering::Relaxed,
        );
    }
}

/// A widget that shows the status of the network.
#[derive(Debug)]
pub struct NetworkWidget {
    status: Arc<NetworkStatus>,
    group: HorizontalWidgetGroup<BinaryColor>,

    widget_local: Rc<RefCell<IconoirWidget>>,
    widget_wifi: Rc<RefCell<IconoirWidget>>,
    widget_internet: Rc<RefCell<IconoirWidget>>,

    quit_send: std::sync::mpsc::Sender<()>,
}

impl NetworkWidget {
    pub fn new() -> Self {
        let status = Arc::new(NetworkStatus::default());
        let (quit_send, quit_recv) = std::sync::mpsc::channel();

        std::thread::spawn({
            let status = status.clone();
            move || loop {
                loop {
                    status.update();

                    if quit_recv
                        .recv_timeout(std::time::Duration::from_secs(3))
                        .is_ok()
                    {
                        break;
                    }
                }
            }
        });

        let widget_local = Rc::new(RefCell::new(IconoirWidget::new::<
            embedded_iconoir::size16px::connectivity::Network,
        >()));
        let widget_wifi = Rc::new(RefCell::new(IconoirWidget::new::<
            embedded_iconoir::size16px::connectivity::Wifi,
        >()));
        let widget_internet = Rc::new(RefCell::new(IconoirWidget::new::<
            embedded_iconoir::size16px::communication::Globe,
        >()));

        let group = HorizontalWidgetGroup::new().with_spacing(1);

        Self {
            status,
            group,
            widget_local,
            widget_wifi,
            widget_internet,
            quit_send,
        }
    }
}

impl Drop for NetworkWidget {
    fn drop(&mut self) {
        self.quit_send.send(()).unwrap();
    }
}

impl Widget for NetworkWidget {
    type Color = BinaryColor;

    fn size_hint(&self, parent_size: Size) -> Size {
        self.group.size_hint(parent_size)
    }

    fn update(&mut self) {
        self.group.clear();
        if self.status.local.load(Ordering::Relaxed) {
            self.group.append(self.widget_local.clone());
        }
        if self.status.wifi.load(Ordering::Relaxed) {
            self.group.append(self.widget_wifi.clone());
        }
        if self.status.internet.load(Ordering::Relaxed) {
            self.group.append(self.widget_internet.clone());
        }
    }

    fn draw(&self, target: &mut DrawBuffer<Self::Color>) {
        self.group.draw(target);
    }
}
