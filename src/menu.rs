#[cfg(feature = "de10")]
extern "C" {
    pub fn HandleUI();
}

#[cfg(not(feature = "de10"))]
pub fn HandleUI() {}
