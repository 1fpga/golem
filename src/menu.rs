#[cfg(feature = "platform_de10")]
extern "C" {
    pub fn HandleUI();
}

#[cfg(not(feature = "platform_de10"))]
pub fn HandleUI() {}
