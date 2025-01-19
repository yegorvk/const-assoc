use const_array_map::{const_array_map, PrimitiveEnum};

macro_rules! enum_tests {
    ($($test_name:ident => $repr:ty),* $(,)?) => {
        $(
            #[test]
            fn $test_name() {
                #[repr($repr)]
                #[derive(Copy, Clone, PrimitiveEnum)]
                enum Letter {
                    A,
                    B,
                    C,
                    D,
                }

                let mut letters = const_array_map! {
                    Letter::A => 'a',
                    Letter::B => 'b',
                    Letter::C => 'c',
                    Letter::D => 'd',
                };

                assert_eq!(letters[Letter::A], 'a');
                assert_eq!(*letters.get(Letter::B), 'b');
                assert_eq!(*letters.const_get(Letter::C), 'c');

                letters[Letter::B] = 'x';
                assert_eq!(letters[Letter::B], 'x');
            }
        )*
    };
}

#[cfg(target_pointer_width = "16")]
enum_tests! {
    enum_u8 => u8,
    enum_u16 => u16,
    enum_usize => usize,
}

#[cfg(target_pointer_width = "32")]
enum_tests! {
    enum_u8 => u8,
    enum_u16 => u16,
    enum_u32 => u32,
    enum_usize => usize,
}

#[cfg(target_pointer_width = "64")]
enum_tests! {
    enum_u8 => u8,
    enum_u16 => u16,
    enum_u32 => u32,
    enum_u64 => u64,
    enum_usize => usize,
}
