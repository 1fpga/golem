#[macro_export]
macro_rules! declare_field_accessors {
    ($(#[$fattr:meta])* $fname: ident, $ftype: ty [padding]) => {};
    ($(#[$fattr:meta])* $fname: ident, $ftype: ty [readonly]) => {
        $(#[$fattr])*
        #[inline]
        pub fn $fname(&self) -> $ftype {
            unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.$fname)) }
        }
    };
    ($(#[$fattr:meta])* $fname: ident, $ftype: ty [writeonly]) => {
        paste::paste! {
            $(#[$fattr])*
            #[inline]
            pub fn [<set_ $fname>](&mut self, value: $ftype) {
                unsafe {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(self.$fname), value);
                }
            }
        }
    };
    ($(#[$fattr:meta])* $fname: ident, $ftype: ty []) => {
        paste::paste! {
            $(#[$fattr])*
            #[inline]
            pub fn $fname(&self) -> $ftype {
                unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.$fname)) }
            }

            $(#[$fattr])*
            #[inline]
            pub fn [<set_ $fname>](&mut self, value: $ftype) {
                unsafe {
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(self.$fname), value);
                }
            }

            paste::paste! {
                $(#[$fattr])*
                #[inline]
                pub fn [<update_ $fname>](&mut self, f: impl FnOnce(&mut $ftype)) {
                    let mut value = self.$fname();
                    f(&mut value);
                    self.[<set_ $fname>](value);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! declare_volatile_struct {
    (
        $(#[$sattr:meta])*
        pub struct $sname: ident {
            $(
                $(#[$fattr:meta])*
                $([$($tags: ident)*])? $fname: ident: $ftype: ty,
            )*
        }
    ) => {
        $(#[$sattr])*
        pub struct $sname {
            $(
                $(#[$fattr])*
                $fname: $ftype,
            )*
        }

        impl $sname {
            $(
                $crate::declare_field_accessors!($(#[$fattr])* $fname, $ftype [$($($tags)*)?]);
            )*
        }
    };
}
