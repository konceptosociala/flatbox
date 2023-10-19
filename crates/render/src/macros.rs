pub use crate::{glenum_wrapper, set_vertex_attribute, c_string};

#[macro_export]
macro_rules! glenum_wrapper {
    {
        wrapper: $wrapper:ident,
        variants: [
            $( $variant:ident ),+
        ]
    } => {
        #[allow(unused_imports)]
        use ::gl::*;

        #[derive(Clone, Copy, Debug)]
        #[repr(u32)]
        pub enum $wrapper {
            $(
                $variant = ::casey::shouty!($variant),
            )+
        }
    };
}

#[macro_export]
macro_rules! c_string {
    ($lit:expr) => {
        {
            ::std::ffi::CString::new($lit).expect("Cannot create CString from literal")
        }
    };
}

#[macro_export]
macro_rules! set_vertex_attribute {
    ($vao:ident, $pos:tt, $t:ident :: $field:tt, $attrib_type:expr) => {
        {
            use core::mem::size_of;
            use $crate::hal::buffer::AttributeType::*;

            const fn size_of_raw<T>(_: *const T) -> usize {
                size_of::<T>()
            }

            let dummy = core::mem::MaybeUninit::<$t>::uninit();
            let dummy_ptr = dummy.as_ptr();
            let member_ptr = unsafe { core::ptr::addr_of!((*dummy_ptr).$field) };
            let member_offset = member_ptr as i32 - dummy_ptr as i32;

            let size = match $attrib_type {
                Byte            => size_of::<i8>(),
                UnsignedByte    => size_of::<u8>(),
                Short           => size_of::<i16>(),
                UnsignedShort   => size_of::<u16>(),
                Int             => size_of::<i32>(),
                UnsignedInt     => size_of::<u32>(),
                Float           => size_of::<f32>(),
                Double          => size_of::<f64>(),
            };

            println!("{}: {}", stringify!($pos), (size_of_raw(member_ptr) / size) as i32);

            unsafe { 
                $vao.set_attribute::<$t>(
                    $pos,
                    $attrib_type,
                    (size_of_raw(member_ptr) / size) as i32,
                    member_offset,
                )
            }
        }
    };
}