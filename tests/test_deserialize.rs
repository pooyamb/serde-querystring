use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use serde_querystring::from_str;

/// It is a helper struct we use to test primitive types
/// as we don't support anything beside maps/structs at the root level
#[derive(Debug, PartialEq, Deserialize)]
struct Primitive<T> {
    value: T,
}

impl<T> Primitive<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

macro_rules! p {
    ($value:expr) => {
        Primitive::new($value)
    };
}

/// Check if all integer types are deserialized
#[test]
fn deserialize_integer_valid() {
    // u8
    assert_eq!(from_str("value=255"), Ok(p!(255_u8)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u8)));

    // i8
    assert_eq!(from_str("value=127"), Ok(p!(127_i8)));
    assert_eq!(from_str("value=-128"), Ok(p!(-128_i8)));

    // u16
    assert_eq!(from_str("value=65535"), Ok(p!(65535_u16)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u16)));

    // i16
    assert_eq!(from_str("value=32767"), Ok(p!(32767_i16)));
    assert_eq!(from_str("value=-32768"), Ok(p!(-32768_i16)));

    // u32
    assert_eq!(from_str("value=4294967295"), Ok(p!(4294967295_u32)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u32)));

    // i32
    assert_eq!(from_str("value=2147483647"), Ok(p!(2147483647_i32)));
    assert_eq!(from_str("value=-2147483648"), Ok(p!(-2147483648_i32)));

    // u64
    assert_eq!(
        from_str("value=18446744073709551615"),
        Ok(p!(18446744073709551615_u64))
    );
    assert_eq!(from_str("value=0"), Ok(p!(0_u64)));

    // i64
    assert_eq!(
        from_str("value=9223372036854775807"),
        Ok(p!(9223372036854775807_i64))
    );
    assert_eq!(
        from_str("value=-9223372036854775808"),
        Ok(p!(-9223372036854775808_i64))
    );
}

/// Check integers overflow
#[test]
fn deserialize_integer_invalid() {
    // u8
    assert!(from_str::<u8>("value=-10").is_err());
    assert!(from_str::<u8>("value=260").is_err());

    // i8
    assert!(from_str::<i8>("value=255").is_err());
    assert!(from_str::<i8>("value=-200").is_err());

    // u16
    assert!(from_str::<u16>("value=65537").is_err());
    assert!(from_str::<u16>("value=-200").is_err());

    // i16
    assert!(from_str::<i16>("value=32768").is_err());
    assert!(from_str::<i16>("value=-32769").is_err());

    // u32
    assert!(from_str::<u32>("value=4294967296").is_err());
    assert!(from_str::<u32>("value=-200").is_err());

    // i32
    assert!(from_str::<i32>("value=2147483648").is_err());
    assert!(from_str::<i32>("value=-2147483649").is_err());

    // u64
    assert!(from_str::<u64>("value=18446744073709551616").is_err());
    assert!(from_str::<u64>("value=-200").is_err());

    // i64
    assert!(from_str::<i64>("value=9223372036854775808").is_err());
    assert!(from_str::<i64>("value=-9223372036854775809").is_err());

    // invalid for integer
    assert!(from_str::<i64>("value=1.5").is_err());
    assert!(from_str::<i64>("value=-1.5").is_err());
    assert!(from_str::<i64>("value=1.2E3").is_err());
    assert!(from_str::<i64>("value=1.2E-3").is_err());
}

/// Check if normal/exponential floats work
#[test]
fn deserialize_float_valid() {
    assert_eq!(from_str("value=1.2"), Ok(p!(1.2_f64)));
    assert_eq!(from_str("value=-1.2"), Ok(p!(-1.2_f64)));
    assert_eq!(from_str("value=1.2E5"), Ok(p!(1.2E5_f64)));
    assert_eq!(from_str("value=-1.2E5"), Ok(p!(-1.2E5_f64)));
    assert_eq!(from_str("value=1.2E+5"), Ok(p!(1.2E5_f64)));
    assert_eq!(from_str("value=-1.2E+5"), Ok(p!(-1.2E5_f64)));
    assert_eq!(from_str("value=1.2E-5"), Ok(p!(1.2E-5_f64)));
    assert_eq!(from_str("value=-1.2E-5"), Ok(p!(-1.2E-5_f64)));
    assert_eq!(
        from_str("value=9223372036854775808"),
        Ok(p!(9223372036854775808_f64))
    );
    assert_eq!(
        from_str("value=-9223372036854775809"),
        Ok(p!(-9223372036854775809_f64))
    );
}

/// Check invalid strings as numbers
#[test]
fn deserialize_float_invalid() {
    assert!(from_str::<f64>("value=number").is_err());
    assert!(from_str::<f64>("value=-1.5num").is_err());
    assert!(from_str::<f64>("value=&").is_err());
    assert!(from_str::<f64>("value=1.0a1.0").is_err());
}

/// Check if different boolean idents work
#[test]
fn deserialize_bool() {
    // true
    assert_eq!(from_str("value=1"), Ok(p!(true)));
    assert_eq!(from_str("value=on"), Ok(p!(true)));
    assert_eq!(from_str("value=true"), Ok(p!(true)));

    // false
    assert_eq!(from_str("value=0"), Ok(p!(false)));
    assert_eq!(from_str("value=off"), Ok(p!(false)));
    assert_eq!(from_str("value=false"), Ok(p!(false)));

    // invalid
    assert!(from_str::<Primitive<bool>>("value=bla").is_err());
    assert!(from_str::<Primitive<bool>>("value=0off").is_err());
    assert!(from_str::<Primitive<bool>>("value=of").is_err());
    assert!(from_str::<Primitive<bool>>("value=onoff").is_err());
}

#[test]
fn deserialize_strings() {
    assert_eq!(from_str("value=test"), Ok(p!("test".to_string())));
    assert_eq!(from_str("value=test"), Ok(p!("test")));
    assert_eq!(from_str("value=250"), Ok(p!("250")));
    assert_eq!(from_str("value=-25"), Ok(p!("-25")));

    // percentage decoded
    assert_eq!(
        from_str("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"),
        Ok(p!("بابابزرگ".to_string()))
    );

    // We can't visit percent decoded strings as &str
    assert!(
        from_str::<Primitive<&str>>("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF")
            .is_err()
    );

    // Plus in strings should be replaced with space
    assert_eq!(from_str("value=rum+rum"), Ok(p!("rum rum".to_string())));

    // We can't visit strings with plus as &str
    assert!(from_str::<Primitive<&str>>("value=rum+rum").is_err());

    // Check if strings don't pass the &
    let mut map = HashMap::new();
    map.insert("num".to_string(), "rum rum".to_string());
    map.insert("yum".to_string(), "ehem ehem".to_string());
    assert_eq!(from_str("num=rum+rum&yum=ehem+ehem"), Ok(map));

    let mut map = HashMap::new();
    map.insert("baba".to_string(), "بابابزرگ".to_string());
    map.insert("amoo".to_string(), "عمو نوروز".to_string());
    assert_eq!(
        from_str(
            "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
            amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2"
        ),
        Ok(map)
    );
}

/// Check if sequence as values work
#[test]
fn deserialize_value_sequence() {
    // Tuples
    assert_eq!(from_str("value=12,3,4"), Ok(p!((12, 3, 4))));
    assert_eq!(
        from_str("value=hallo,hello,hi"),
        Ok(p!((
            "hallo".to_string(),
            "hello".to_string(),
            "hi".to_string()
        )))
    );

    // Vectors
    assert_eq!(from_str("value=12,3,4"), Ok(p!(vec![12, 3, 4])));
    assert_eq!(
        from_str("value=hallo,hello,hi"),
        Ok(p!(vec!["hallo", "hello", "hi"]))
    );

    // Check if percentage decoded values don't pass ,
    assert_eq!(
        from_str(
            "value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF+,\
            %D8%B9%D9%85%D9%88+%D9%86%D9%88%D8%B1%D9%88%D8%B2,\
            %D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"
        ),
        Ok(p!(vec![
            "بابابزرگ ".to_string(),
            "عمو نوروز".to_string(),
            "بابابزرگ".to_string()
        ]))
    );

    // Tuples with extra values should not work
    assert!(from_str::<Primitive<(i32, i32, i32)>>("value=12,3,4,32").is_err());
    assert!(from_str::<Primitive<(&str, &str, &str)>>("value=hallo,hello,hi,bla").is_err());

    // We don't support sequences of sequences(may support in future for tuples)
    assert!(from_str::<Primitive<((i32, i32), (i32, i32))>>("value=12,3,4,32").is_err());
    assert!(from_str::<Primitive<Vec<Vec<i32>>>>("value=12,3,4,32").is_err());
}

#[test]
fn deserialize_new_type() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct NewType(i32);

    assert_eq!(from_str("value=-2500000"), Ok(p!(NewType(-2_500_000))));
}

/// Check if unit enums work as keys, values and sequence as values
#[test]
fn deserialize_unit_enums() {
    // as key
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    enum Side {
        Left,
        Right,
        God,
    }

    assert_eq!(from_str("value=God"), Ok(p!(Side::God)));

    let mut map = HashMap::new();
    map.insert(Side::God, "winner");
    map.insert(Side::Right, "looser");
    assert_eq!(from_str("God=winner&Right=looser"), Ok(map));

    // as value
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        from_str::<A>("looser=Left&winner=God"),
        Ok(A {
            looser: Side::Left,
            winner: Side::God
        })
    );

    // as subkey or sub value
    #[derive(Debug, Deserialize, PartialEq)]
    struct B {
        sides: Vec<Side>,
        result: HashMap<Side, i32>,
    }
    let mut map = HashMap::new();
    map.insert(Side::God, 10);
    map.insert(Side::Right, -1);
    map.insert(Side::Left, -1);
    assert_eq!(
        from_str::<B>("sides=God,Left,Right&result[God]=10&result[Right]=-1&result[Left]=-1"),
        Ok(B {
            sides: vec![Side::God, Side::Left, Side::Right],
            result: map
        })
    );
}

#[test]
fn deserialize_map() {
    let mut res = HashMap::new();
    res.insert("key1".to_string(), 321);
    res.insert("key2".to_string(), 123);
    res.insert("key3".to_string(), 7);
    res.insert("key4".to_string(), 6);
    assert_eq!(from_str("key1=321&key2=123&key3=7&key4=6"), Ok(res));
}

#[test]
fn deserialize_map_of_maps() {
    fn hashmap_it<T>(t: T) -> HashMap<String, T> {
        let mut map = HashMap::new();
        map.insert("key".to_string(), t);
        map
    }

    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());

    // 10 times
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);

    // 10 more times
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let map = hashmap_it(map);
    let ok_map = Ok(map);

    assert_eq!(
        from_str(
            "key[key][key][key][key][key][key][key][key][key][key][key]\
             [key][key][key][key][key][key][key][key][key]=value",
        ),
        ok_map
    );
}

#[test]
fn deserialize_map_with_repeated_keys() {
    let mut map = HashMap::new();
    map.insert("num".to_string(), -2501);
    assert_eq!(
        from_str::<HashMap<String, i32>>("num=-2500&num=-2503&num=-2502&num=-2501"),
        Ok(map)
    );
}

/// We already tested simple structs in all primitive value tests, check if multiple fields
/// of different values also work and don't overflow on each other
#[test]
fn deserialize_struct() {
    #[derive(Debug, serde::Serialize, Deserialize, Eq, PartialEq)]
    struct Sample<'a> {
        neg_num: i32,
        num: u8,
        string: String,
        strings: Vec<String>,
        #[serde(borrow)]
        one_str: &'a str,
        #[serde(borrow)]
        strs: Vec<&'a str>,
        #[serde(borrow)]
        strs_tuple: (&'a str, &'a str, &'a str),
        boolean: bool,
        booleans: Vec<bool>,
    }

    let s = Sample {
        neg_num: -2500,
        num: 123,
        one_str: "HiBabe",
        string: "بابابزرگ &".to_string(),
        strings: vec![
            "بابابزرگ ".to_string(),
            "عمو نوروز,".to_string(),
            "بابابزرگ&".to_string(),
        ],
        strs: vec!["Hello", "World", "Its", "Me"],
        strs_tuple: ("Hello", "World", "ITSME"),
        boolean: true,
        booleans: vec![false, true, false, true, false, true, false],
    };

    assert_eq!(
        from_str::<Sample>(
            "neg_num=-2500&\
            string=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF+%26&\
            strings=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF+,\
            %D8%B9%D9%85%D9%88+%D9%86%D9%88%D8%B1%D9%88%D8%B2%2C,\
            %D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF%26&\
            one_str=HiBabe&strs=Hello,World,Its,Me&\
            strs_tuple=Hello,World,ITSME&\
            boolean=true&booleans=false,true,off,on,0,1,false&\
            num=123"
        ),
        Ok(s)
    );
}

#[test]
fn deserialize_struct_of_structs() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Book {
        pages: usize,
        finished: bool,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Child {
        age: i32,
        book: Book,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Human {
        child: Child,
        book: Book,
        name: String,
    }

    let human = Ok(Human {
        child: Child {
            age: 12,
            book: Book {
                pages: 1000,
                finished: false,
            },
        },
        book: Book {
            pages: 300,
            finished: true,
        },
        name: "Regina Phalange".to_string(),
    });

    assert_eq!(
        from_str::<Human>(
            "child[age]=12&child[reads]=on&name=Regina+Phalange&book[pages]=300&book[finished]=true\
            &child[book][pages]=1000&child[book][finished]=off"
        ),
        human
    );

    assert_eq!(
        from_str::<Human>(
            "name=Regina+Phalange&child[age]=12&child[reads]=on&book[pages]=300\
            &book[finished]=true&child[book][pages]=1000&child[book][finished]=off"
        ),
        human
    );
}

#[test]
fn deserialize_enums() {
    // from rust by example book
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    enum Event {
        #[serde(rename = "بارگذاری صفحه")]
        PageLoad,
        PageUnload,
        KeyPress(char),
        Paste(String),
        Click {
            x: i64,
            y: i64,
        },
        Missed(i32, i32),
    }

    use Event::*;

    assert_eq!(
        from_str(
            "value=%D8%A8%D8%A7%D8%B1%DA%AF%D8%B0%D8%A7%D8%B1%DB%8C%20%D8%B5%D9%81%D8%AD%D9%87"
        ),
        Ok(p!(PageLoad))
    );
    assert_eq!(from_str("value=PageUnload"), Ok(p!(PageUnload)));
    assert_eq!(
        from_str("value=%D8%A8%D8%A7%D8%B1%DA%AF%D8%B0%D8%A7%D8%B1%DB%8C%20%D8%B5%D9%81%D8%AD%D9%87,PageUnload"),
        Ok(p!((PageLoad, PageUnload)))
    );
    assert_eq!(
        from_str(
            "value[%D8%A8%D8%A7%D8%B1%DA%AF%D8%B0%D8%A7%D8%B1%DB%8C%20%D8%B5%D9%81%D8%AD%D9%87]="
        ),
        Ok(p!(PageLoad))
    );
    assert_eq!(from_str("value[PageUnload]="), Ok(p!(PageUnload)));
    assert_eq!(from_str("value[KeyPress]=2"), Ok(p!(KeyPress('2'))));
    assert_eq!(
        from_str("value[Paste]=asd"),
        Ok(p!(Paste("asd".to_string())))
    );
    assert_eq!(
        from_str("value[Missed]=1200,2400"),
        Ok(p!(Missed(1200, 2400)))
    );
    assert_eq!(
        from_str("value[Missed][]=1200&value[Missed][]=2400"),
        Ok(p!(Missed(1200, 2400)))
    );
    assert_eq!(
        from_str("value[Missed][2]=2400&value[Missed][1]=1200"),
        Ok(p!(Missed(1200, 2400)))
    );
    assert_eq!(
        from_str("value[Click][x]=5500&value[Click][y]=6900"),
        Ok(p!(Click { x: 5500, y: 6900 }))
    );

    // struct and tuple enums
    assert_eq!(
        from_str(
            "value[][Paste]=asd&value[][Missed]=1200,2400\
            &value[group_1][Click][x]=5500&value[group_1][Click][y]=6900"
        ),
        Ok(p!(vec![
            Paste("asd".to_string()),
            Missed(1200, 2400),
            Click { x: 5500, y: 6900 },
        ]))
    );
    assert_eq!(
        from_str("value[][Missed]=1200,2400&value[][Missed]=1200,2400&value[][Paste]=asd"),
        Ok(p!(vec![
            Missed(1200, 2400),
            Missed(1200, 2400),
            Paste("asd".to_string())
        ]))
    );
}

#[test]
fn deserialize_sequence() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct UvRate {
        nums: Vec<i32>,
        average: i32,
    }
    assert_eq!(
        from_str::<UvRate>("nums[]=1&nums[]=3&nums[]=1337&average=447"),
        Ok(UvRate {
            nums: vec![1, 3, 1337],
            average: 447
        })
    );

    assert_eq!(
        from_str::<UvRate>("nums[0]=1&nums[1]=2&nums[2]=1337&average=300"),
        Ok(UvRate {
            nums: vec![1, 2, 1337],
            average: 300
        })
    );

    #[derive(Debug, Deserialize, PartialEq)]
    struct Country {
        sun: UvRate,
    }

    assert_eq!(
        from_str::<Country>("sun[nums][]=1&sun[nums][]=3&sun[nums][]=1337&sun[average]=447"),
        Ok(Country {
            sun: UvRate {
                nums: vec![1, 3, 1337],
                average: 447
            }
        })
    );

    assert_eq!(
        from_str::<Country>("sun[nums][0]=1&sun[nums][1]=2&sun[nums][2]=1337&sun[average]=300"),
        Ok(Country {
            sun: UvRate {
                nums: vec![1, 2, 1337],
                average: 300
            }
        })
    );

    #[derive(Debug, Deserialize, PartialEq)]
    enum Weather {
        Sunny { uv: usize, tempt: usize },
        Rainy(char, char),
        Hot(usize),
        Cold,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct City {
        history: Vec<Weather>,
    }

    assert_eq!(
        from_str::<City>(
            "history[0]=Cold&history[1][Rainy]=a,b&history[2][Sunny][uv]=100&history[2][Sunny][tempt]=100&\
            history[3][Hot]=10&history[4][Rainy][]=k&history[4][Rainy][]=o"
        ),
        Ok(City { history: vec![
            Weather::Cold,
            Weather::Rainy('a', 'b'),
            Weather::Sunny { uv: 100, tempt: 100 },
            Weather::Hot(10),
            Weather::Rainy('k', 'o'),
        ] })
    );
}

#[test]
fn deserialize_sequence_ordered() {
    assert_eq!(
        from_str("value[10]=9&value[10]=10&value[1]=2&value[8]=8&value[2]=3&value[6]=7&value[0]=1&value[]=0"),
        Ok(p!(vec![0, 1, 2, 3, 7, 8, 10]))
    );

    #[derive(Debug, Deserialize, PartialEq)]
    enum Anum {
        Q,
        W(i32),
        E(i32, i32),
        R { x: i32, y: i32 },
    }

    // Keyed sequences are last defined group
    assert_eq!(
        from_str("value[1][E][]=1&value[1][R][y]=2&value[1][E][]=2&value[1][R][x]=1"),
        Ok(p!(vec![Anum::R { x: 1, y: 2 }]))
    );

    // Exceptional enum in sequence
    assert_eq!(from_str("value[1]=Q&value[1][W]=10"), Ok(p!(vec![Anum::Q])));
    assert_eq!(from_str("value[1][W]=10&value[1]=Q"), Ok(p!(vec![Anum::Q])));
    assert_eq!(
        from_str("value[1][Q]=&value[1][W]=10"),
        Ok(p!(vec![Anum::W(10)]))
    );
}

#[test]
fn deserialize_string_key() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Family {
        #[serde(rename = "بابا")]
        dad: String,
        #[serde(rename = "مامان")]
        mom: String,
        #[serde(rename = "مامان بزرگ")]
        grandma: String,
    }

    assert_eq!(
        from_str(
            "%D8%A8%D8%A7%D8%A8%D8%A7=%D9%BE%D8%AF%D8%B1&\
            %D9%85%D8%A7%D9%85%D8%A7%D9%86=%D9%85%D8%A7%D8%AF%D8%B1&\
            %D9%85%D8%A7%D9%85%D8%A7%D9%86+%D8%A8%D8%B2%D8%B1%DA%AF=\
            %D9%85%D8%A7%D8%AF%D8%B1+%D8%A8%D8%B2%D8%B1%DA%AF"
        ),
        Ok(Family {
            dad: "پدر".to_string(),
            mom: "مادر".to_string(),
            grandma: "مادر بزرگ".to_string()
        }),
    );
}

#[test]
fn deserialize_number_key() {
    let mut map = HashMap::new();
    map.insert(1, "Only");
    map.insert(2, "Couple");
    map.insert(3, "Some");

    assert_eq!(from_str("1=Only&2=Couple&3=Some"), Ok(map));
}

#[test]
fn deserialize_sequence_as_value_key() {
    #[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
    struct Point(i32, i32, i32);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        weight: HashMap<Point, i32>,
    };

    let mut weight = HashMap::new();
    weight.insert(Point(1, 1, 1), 1200);
    weight.insert(Point(2, 2, 2), 2400);

    let res = Ok(Sample { weight });

    assert_eq!(from_str("weight[1,1,1]=1200&weight[2,2,2]=2400"), res);

    // TODO: This should not work
    assert_eq!(from_str("weight[1,1,1,1]=1200&weight[2,2,2,2]=2400"), res);

    // The same with strings
    #[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
    struct PointString(String, String, String);

    #[derive(Debug, Deserialize, PartialEq)]
    struct SampleString {
        weight: HashMap<PointString, String>,
    };

    let mut weight = HashMap::new();
    weight.insert(
        PointString("1a".to_string(), "1a".to_string(), "1a".to_string()),
        "big".to_string(),
    );
    weight.insert(
        PointString("2a".to_string(), "2a".to_string(), "2a".to_string()),
        "small".to_string(),
    );

    let res = Ok(SampleString { weight });

    assert_eq!(from_str("weight[1a,1a,1a]=big&weight[2a,2a,2a]=small"), res);

    // TODO: this should not work
    assert_eq!(
        from_str("weight[1a,1a,1a,1a]=big&weight[2a,2a,2a,2a]=small"),
        res
    );
}

// We don't support these kind of recursive structures
#[test]
fn deserialize_invalid_recursion() {
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct A {
        a: BTreeMap<i32, A>,
    };

    assert!(from_str::<A>("a[2]=a[2]=").is_err());

    #[derive(Debug, Clone, Deserialize)]
    struct A2(Vec<A2>);

    assert!(from_str::<Primitive<A2>>("value=1,1,1").is_err())
}

#[test]
fn deserialize_recursion_overflow() {
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct B {
        a: Box<Option<B>>,
    };

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct A {
        a: B,
    };

    // An input of `a[a][a][a][a]=` is valid in this case as long as we stay below the recurstion limit
    let mut s = "a".to_string();
    for _ in 0..64 {
        s.push_str("[a]");
    }
    s.push('=');

    assert!(from_str::<A>(&s).is_ok());

    let mut s = "a".to_string();
    for _ in 0..65 {
        s.push_str("[a]");
    }
    s.push('=');

    assert!(from_str::<A>(&s).is_err());
}

#[test]
fn deserialize_without_and() {
    assert!(from_str::<HashMap<String, i32>>("key1=321key2=123key3=7").is_err());

    let mut map = HashMap::new();
    map.insert("key1".to_string(), "321key2=123key3=7".to_string());
    assert_eq!(
        from_str::<HashMap<String, String>>("key1=321key2=123key3=7"),
        Ok(map)
    );
}

#[test]
fn deserialize_unit_value() {
    assert_eq!(from_str(""), Ok(()));
    assert_eq!(from_str("&"), Ok(()));
    assert_eq!(from_str("&&"), Ok(()));
    assert_eq!(from_str("&&&;;;"), Ok(()));
    assert!(from_str::<()>("&&&;;;a=b").is_err());
}

#[test]
fn deserialize_bench_child() {
    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
    struct SampleChild {
        x: i32,
        y: i32,
        z: i32,
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
    struct Sample {
        x: SampleChild,
        y: SampleChild,
        z: SampleChild,
    }
    let unordered = "z[z]=33333&z[y]=222222&z[x]=11111&\
                     y[z]=33333&y[y]=222222&y[x]=11111&\
                     x[z]=33333&x[y]=222222&x[x]=11111";
    let ordered = "x[x]=11111&x[y]=222222&x[z]=33333&\
                    y[x]=11111&y[y]=222222&y[z]=33333&\
                    z[x]=11111&z[y]=222222&z[z]=33333";

    let child = SampleChild {
        x: 11111,
        y: 222222,
        z: 33333,
    };

    let res = Sample {
        x: child,
        y: child,
        z: child,
    };

    // Check if everything is working as expected
    assert_eq!(from_str(&ordered), Ok(res));
    assert_eq!(from_str(&unordered), Ok(res));
}

#[test]
fn deserialize_bench_decoded() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        amoo: String,
        baba: String,
    }
    let ordered = "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
                   amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2";
    let unordered = "amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2&\
                    baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF";

    let res = Ok(Sample {
        amoo: "عمو نوروز".to_string(),
        baba: "بابابزرگ".to_string(),
    });

    assert_eq!(from_str::<Sample>(ordered), res);
    assert_eq!(from_str::<Sample>(unordered), res);
}

#[test]
fn deserialize_bench_multilevel() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Level4 {
        x4: String,
        y4: String,
        z4: String,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Level3 {
        x3: Level4,
        y3: Level4,
        z3: Level4,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Level2 {
        x2: Level3,
        y2: Level3,
        z2: Level3,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Level1 {
        x1: Level2,
        y1: Level2,
        z1: Level2,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Sample {
        x0: Level1,
        y0: Level1,
        z0: Level1,
    }

    let ordered = include_str!("multilevel_ordered.txt");
    let unordered = include_str!("multilevel_unordered.txt");

    // Just checking them to be equal and without errors
    assert_eq!(
        from_str::<Sample>(ordered).unwrap(),
        from_str::<Sample>(unordered).unwrap()
    );
}

#[derive(Deserialize, PartialEq)]
struct SeqStruct {
    value: Vec<i32>,
}

impl std::fmt::Debug for SeqStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Long sequence of values").finish()
    }
}

#[test]
fn deserialize_bench_seq() {
    let mut ordered = String::new();
    let mut res_ordered = Vec::new();
    for i in 0..1000 {
        ordered.push_str(&format!("value[{}]={}&", i, 1024 * i));
        res_ordered.push(1024 * i);
    }
    ordered.remove(ordered.len() - 1);

    let mut reverse = String::new();
    for i in 1..=1000 {
        reverse.push_str(&format!("value[{}]={}&", 1000 - i, 1024 * (1000 - i)));
    }
    reverse.remove(reverse.len() - 1);

    assert_eq!(
        from_str(&ordered),
        Ok(SeqStruct {
            value: res_ordered.clone()
        })
    );
    assert_eq!(from_str(&reverse), Ok(SeqStruct { value: res_ordered }));
}

#[test]
fn deserialize_invalid() {
    // from_str::<HashMap<String, Vec<i32>>>("x[3]=22&&x[2]")
}

#[test]
fn deserialize_to_unit() {
    // from_str::<HashMap<String, ()>>("x[3]=22&x[2]=22") also for struct fields
}
