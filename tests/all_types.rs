#![allow(warnings)]
use cor::Encoder;

#[derive(Encoder, Clone)]
struct Types<'a> {
    #[key = 1]
    bool_true: bool,
    #[key = 2]
    bool_false: bool,

    #[key = 3]
    u16_min: u16,
    #[key = 4]
    u16_max: u16,

    #[key = 5]
    i16_min: i16,
    #[key = 6]
    i16_max: i16,

    #[key = 7]
    u32_min: u32,
    #[key = 8]
    u32_max: u32,

    #[key = 9]
    i32_min: i32,
    #[key = 10]
    i32_max: i32,

    #[key = 11]
    u64_min: u64,
    #[key = 12]
    u64_max: u64,

    #[key = 13]
    i64_min: i64,
    #[key = 14]
    i64_max: i64,

    #[key = 15]
    f32_min: f32,
    #[key = 16]
    f32_max: f32,

    #[key = 17]
    f64_min: f64,
    #[key = 18]
    f64_max: f64,

    #[key = 19]
    string: &'a str,
    #[key = 20]
    bytes: &'a [u8],

    #[key = 21]
    user: User,
}

impl<'de> cor::Decoder<'de> for Types<'de> {
    fn decode(e: &cor::Entries<'de>) -> std::io::Result<Self> {
        Ok(Self {
            bool_true: todo!(),
            bool_false: todo!(),
            u16_min: todo!(),
            u16_max: todo!(),
            i16_min: todo!(),
            i16_max: todo!(),
            u32_min: todo!(),
            u32_max: todo!(),
            i32_min: todo!(),
            i32_max: todo!(),
            u64_min: todo!(),
            u64_max: todo!(),
            i64_min: todo!(),
            i64_max: todo!(),
            f32_min: todo!(),
            f32_max: todo!(),
            f64_min: todo!(),
            f64_max: todo!(),
            string: todo!(),
            bytes: todo!(),
            user: todo!(),
        })
    }
}

#[derive(Encoder, Debug, Clone)]
struct User {
    #[key = 0]
    id: Vec<u8>,
    #[key = 1]
    name: String,
    #[key = 2]
    email: Option<String>,
}

#[test]
fn test_all_types() {
    let types = Types {
        bool_true: true,
        bool_false: false,

        u16_min: 0,
        u16_max: u16::MAX,

        i16_min: i16::MIN,
        i16_max: i16::MAX,

        u32_min: 0,
        u32_max: u32::MAX,

        i32_min: i32::MIN,
        i32_max: i32::MAX,

        u64_min: 0,
        u64_max: u64::MAX,

        i64_min: i64::MIN,
        i64_max: i64::MAX,

        f32_min: f32::MIN,
        f32_max: f32::MAX,

        f64_min: f64::MIN,
        f64_max: f64::MAX,

        string: "Hello World",
        bytes: "Hello, World".as_bytes(),
        user: User {
            id: vec![1, 2, 3, 4, 5],
            name: "Alex".into(),
            email: None,
        },
    };

    let mut buf = Vec::new();
    types.encode(&mut buf).unwrap();
    // println!("buf: {:#?}", buf);

    let mut reader = &buf[..];
    let val = cor::Entries::parse(&mut reader).unwrap();
    println!("{:#?}", cor::Value::Struct(val));
}
