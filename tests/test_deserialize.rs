use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use serde_querystring::from_str;

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

#[test]
fn deserialize_primitive_types() {
    assert_eq!(from_str("value=-2500000"), Ok(p!(-2_500_000)));
    assert_eq!(from_str("value=2500000"), Ok(p!(2_500_000)));
    assert_eq!(from_str("value=test"), Ok(p!("test".to_string())));
    assert_eq!(from_str("value=test"), Ok(p!("test")));
    assert_eq!(from_str("value=250"), Ok(p!("250".to_string())));
    assert_eq!(from_str("value=-25"), Ok(p!("-25".to_string())));
    assert_eq!(from_str("value=-1.2222"), Ok(p!(-1.2222)));
    assert_eq!(from_str("value=1.2222"), Ok(p!(1.2222)));
}

#[test]
fn deserialize_bool() {
    assert_eq!(from_str("value=1"), Ok(p!(true)));
    assert_eq!(from_str("value=on"), Ok(p!(true)));
    assert_eq!(from_str("value=true"), Ok(p!(true)));
    assert_eq!(from_str("value=0"), Ok(p!(false)));
    assert_eq!(from_str("value=off"), Ok(p!(false)));
    assert_eq!(from_str("value=false"), Ok(p!(false)));
    assert!(from_str::<Primitive<bool>>("value=bla").is_err());
}

#[test]
fn deserialize_unit_type() {
    assert_eq!(from_str(""), Ok(()));
    assert_eq!(from_str("&"), Ok(()));
    assert_eq!(from_str("&&"), Ok(()));
    assert_eq!(from_str("&&&"), Ok(()));
}

// #[test]
// fn deserialize_with_extra_unit_chars() {
//     assert_eq!(from_str("&value=200"), Ok(((), p!(200))));
//     assert_eq!(from_str("&&value=-200"), Ok(((), (), p!(-200))));
//     assert_eq!(
//         from_str("&&value=test"),
//         Ok(((), (), p!("test".to_string())))
//     );
// }

#[test]
fn deserialize_tuple() {
    assert_eq!(from_str("value=12,3,4"), Ok(p!((12, 3, 4))));
    assert_eq!(from_str("value=12,3,4,32"), Ok(p!((12, 3, 4))));

    assert_eq!(
        from_str("value=hallo,hello,hi"),
        Ok(p!((
            "hallo".to_string(),
            "hello".to_string(),
            "hi".to_string()
        )))
    );
    assert_eq!(
        from_str("value=hallo,hello,hi,bla"),
        Ok(p!((
            "hallo".to_string(),
            "hello".to_string(),
            "hi".to_string()
        )))
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
fn deserialize_new_type() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct NewType(i32);

    assert_eq!(from_str("value=-2500000"), Ok(p!(NewType(-2_500_000))));
}

#[test]
fn deserialize_struct() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Sample {
        neg_num: i32,
        num: u8,
        string: String,
        boolean: bool,
        special_bool: bool,
    }

    let s = Sample {
        neg_num: -2500,
        num: 123,
        string: "Hello".to_string(),
        boolean: true,
        special_bool: false,
    };

    assert_eq!(
        from_str::<Sample>("neg_num=-2500&num=123&string=Hello&boolean=on&special_bool=false"),
        Ok(s)
    );
}

#[test]
fn deserialize_repeated_keys() {
    let mut map = HashMap::new();
    map.insert("num".to_string(), -2501);
    assert_eq!(
        from_str::<HashMap<String, i32>>("num=-2500&num=-2503&num=-2502&num=-2501"),
        Ok(map)
    );
}

#[test]
fn deserialize_plus_to_space() {
    let mut map = HashMap::new();
    map.insert("num".to_string(), "rum rum".to_string());
    map.insert("yum".to_string(), "ehem ehem".to_string());
    assert_eq!(
        from_str::<HashMap<String, String>>("num=rum+rum&yum=ehem+ehem"),
        Ok(map)
    );
}

#[test]
fn deserialize_percentage_decoding() {
    let mut map = HashMap::new();
    map.insert("baba".to_string(), "بابابزرگ".to_string());
    map.insert("amoo".to_string(), "عمو نوروز".to_string());
    assert_eq!(
        from_str::<HashMap<String, String>>(
            "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
            amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2"
        ),
        Ok(map)
    );
}

#[test]
fn deserialize_percentage_decoding_raw() {
    assert_eq!(
        from_str("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"),
        Ok(p!("بابابزرگ".to_string()))
    );
}

// It should be here, but it is not to keep the same behaviour as serde_urlencoded
// #[test]
// fn deserialize_percentage_invalid() {
//     assert!(from_str::<String>("%00%01%11").is_err());
// }

#[test]
fn deserialize_struct_with_non_primitive_chilren() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct HumanAge(u8);

    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct Human {
        abblities: Vec<String>,
        favs: (i32, i32, i32),
        age: HumanAge,
    }

    let human = Human {
        abblities: vec!["eats".to_string(), "reads".to_string(), "cries".to_string()],
        favs: (1, 2, 3),
        age: HumanAge(69),
    };

    assert_eq!(
        from_str::<Human>("favs=1,2,3&abblities=eats,reads,cries&age=69"),
        Ok(human)
    );
}

#[test]
fn deserialize_struct_with_struct_chilren() {
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
fn deserialize_so_many_levels() {
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

    let deserialized = from_str(
        "key[key][key][key][key][key][key][key][key][key][key][key]\
        [key][key][key][key][key][key][key][key][key]=value",
    )
    .unwrap();

    assert_eq!(map, deserialized);
}

#[test]
fn deserialize_tuple_key() {
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

    // This should also work as we ignore everything after the last ,
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

    // This should also work as we ignore everything after the last ,
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

    // An input of `a[a][a][a][a]=` is valid in this case, but as long as we stay below the recurstion limit
    let mut s = "a".to_string();
    for _ in 0..63 {
        s.push_str("[a]");
    }
    s.push_str("=");

    assert!(from_str::<A>(&s).is_ok());

    let mut s = "a".to_string();
    for _ in 0..64 {
        s.push_str("[a]");
    }
    s.push_str("=");

    assert!(from_str::<A>(&s).is_err());
}

#[test]
fn deserialize_number_overflow() {
    let max_u32_plus_1 = (u32::max_value() as i64) + 1;
    assert!(from_str::<Primitive<u32>>(&format!("value={}", max_u32_plus_1)).is_err());

    let max_i32_neg_minus_one = -(max_u32_plus_1 / 2) - 1;
    assert!(from_str::<Primitive<i32>>(&format!("value={}", max_i32_neg_minus_one)).is_err());

    assert!(from_str::<Primitive<u64>>("value=18446744073709551616").is_err());

    assert!(from_str::<Primitive<i64>>("value=-9223372036854775809").is_err());

    assert_eq!(
        from_str("value=18446744073709551616"),
        Ok(p!(18446744073709551616_f64))
    );
    assert_eq!(
        from_str("value=-18446744073709551616"),
        Ok(p!(-18446744073709551616_f64))
    );
}

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
    assert_eq!(
        from_str::<HashMap<Side, &str>>("God=winner&Right=looser"),
        Ok(map)
    );

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
fn deserialize_enum_in_map() {
    // from rust by example book
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    enum Event {
        PageLoad,
        PageUnload,
        KeyPress(char),
        Paste(String),
        Click { x: i64, y: i64 },
        Missed(i32, i32),
    }

    use Event::*;

    assert_eq!(from_str("value=PageLoad"), Ok(p!(PageLoad)));
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
        from_str("value[Click][x]=5500&value[Click][y]=6900"),
        Ok(p!(Click { x: 5500, y: 6900 }))
    );
}

#[test]
fn deserialize_enums() {
    // from rust by example book
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    enum Event {
        PageLoad,
        PageUnload,
        KeyPress(char),
        Paste(String),
        Click { x: i64, y: i64 },
        Missed(i32, i32),
    }

    // deserialize to enum itself
    // unit
    assert_eq!(from_str::<Event>("PageUnload"), Ok(PageUnload));

    // sequence
    assert_eq!(
        from_str::<Event>("Missed=400,640"),
        Ok(Event::Missed(400, 640))
    );
    assert_eq!(
        from_str::<Event>("Missed[]=400&Missed[]=640"),
        Ok(Event::Missed(400, 640))
    );
    assert_eq!(
        from_str::<Event>("Missed[1]=640&Missed[0]=400"),
        Ok(Event::Missed(400, 640))
    );

    // struct
    assert_eq!(
        from_str::<Event>("Click[x]=100&Click[y]=240"),
        Ok(Event::Click { x: 100, y: 240 })
    );

    // new type
    assert_eq!(from_str::<Event>("KeyPress=X"), Ok(Event::KeyPress('X')));

    use Event::*;

    // unit enums
    assert_eq!(
        from_str("value=PageLoad,PageUnload"),
        Ok(p!(vec![PageLoad, PageUnload]))
    );
    assert_eq!(
        from_str("value[]=PageLoad&value[]=PageUnload"),
        Ok(p!(vec![PageLoad, PageUnload]))
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
fn deserialize_sequence_key() {
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
fn deserialize_sequence_key_ordered() {
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

    // Keyed sequences are first come first serve
    assert_eq!(
        from_str("value[1][E][]=1&value[1][R][y]=2&value[1][E][]=2&value[1][R][x]=1"),
        Ok(p!(vec![Anum::R { x: 1, y: 2 }]))
    );
}

#[test]
fn deserialize_invalid() {
    // from_str::<HashMap<String, Vec<i32>>>("x[3]=22&&x[2]")
}

#[test]
fn deserialize_to_unit() {
    // from_str::<HashMap<String, ()>>("x[3]=22&x[2]=22") also for struct fields
}
