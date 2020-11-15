use std::collections::HashMap;

use serde::Deserialize;

#[test]
fn deserialize_primitive_types() {
    assert_eq!(serde_querystring::from_str("-2500000"), Ok(-2_500_000));
    assert_eq!(serde_querystring::from_str("2500000"), Ok(2_500_000));
    assert_eq!(serde_querystring::from_str("test"), Ok("test".to_string()));
    assert_eq!(serde_querystring::from_str("test"), Ok("test"));
    assert_eq!(serde_querystring::from_str("250"), Ok("250".to_string()));
    assert_eq!(serde_querystring::from_str("-25"), Ok("-25".to_string()));
    assert_eq!(serde_querystring::from_str("-1.2222"), Ok(-1.2222));
    assert_eq!(serde_querystring::from_str("1.2222"), Ok(1.2222));
}

#[test]
fn deserialize_bool() {
    assert_eq!(serde_querystring::from_str("1"), Ok(true));
    assert_eq!(serde_querystring::from_str("on"), Ok(true));
    assert_eq!(serde_querystring::from_str("true"), Ok(true));
    assert_eq!(serde_querystring::from_str("0"), Ok(false));
    assert_eq!(serde_querystring::from_str("off"), Ok(false));
    assert_eq!(serde_querystring::from_str("false"), Ok(false));
    assert!(serde_querystring::from_str::<bool>("bla").is_err());
}

#[test]
fn deserialize_unit_type() {
    assert_eq!(serde_querystring::from_str(""), Ok(()));
    assert_eq!(serde_querystring::from_str("&"), Ok(()));
    assert_eq!(serde_querystring::from_str("&&"), Ok(()));
    assert_eq!(serde_querystring::from_str("&&&"), Ok(()));
}

#[test]
fn deserialize_with_extra_unit_chars() {
    assert_eq!(serde_querystring::from_str("&200"), Ok(((), 200)));
    assert_eq!(serde_querystring::from_str("&&-200"), Ok(((), (), -200)));
    assert_eq!(
        serde_querystring::from_str("&&test"),
        Ok(((), (), "test".to_string()))
    );
}

#[test]
fn deserialize_sequence() {
    let res = vec![12, 3, 4, 5, 6, -10];
    assert_eq!(
        serde_querystring::from_str("12,3,4,5,6,-10"),
        Ok(res.clone())
    );
    assert_eq!(
        serde_querystring::from_str("12&3&4&5&6&-10"),
        Ok(res.clone())
    );
    assert_eq!(serde_querystring::from_str("12;3;4;5;6;-10"), Ok(res));

    let res = vec!["asd".to_string(), "dsa".to_string(), "sss".to_string()];
    assert_eq!(serde_querystring::from_str("asd&dsa&sss"), Ok(res));
}

#[test]
fn deserialize_tuple() {
    assert_eq!(
        serde_querystring::from_str::<(i32, i32, i32)>("12,3,4"),
        Ok((12, 3, 4))
    );
    assert_eq!(
        serde_querystring::from_str::<(i32, i32, i32)>("12,3,4,32"),
        Ok((12, 3, 4))
    );

    assert_eq!(
        serde_querystring::from_str::<(String, String, String)>("hallo,hello,hi"),
        Ok(("hallo".to_string(), "hello".to_string(), "hi".to_string()))
    );
    assert_eq!(
        serde_querystring::from_str::<(String, String, String)>("hallo,hello,hi,bla"),
        Ok(("hallo".to_string(), "hello".to_string(), "hi".to_string()))
    );
}

#[test]
fn deserialize_map() {
    let mut res = HashMap::new();
    res.insert("key1".to_string(), 321);
    res.insert("key2".to_string(), 123);
    res.insert("key3".to_string(), 7);
    res.insert("key4".to_string(), 6);
    assert_eq!(
        serde_querystring::from_str("key1=321&key2=123&key3=7&key4=6"),
        Ok(res)
    );
}

#[test]
fn deserialize_without_and() {
    assert!(serde_querystring::from_str::<HashMap<String, i32>>("key1=321key2=123key3=7").is_err());

    let mut map = HashMap::new();
    map.insert("key1".to_string(), "321key2=123key3=7".to_string());
    assert_eq!(
        serde_querystring::from_str::<HashMap<String, String>>("key1=321key2=123key3=7"),
        Ok(map)
    );
}

#[test]
fn deserialize_new_type() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    struct NewType(i32);

    assert_eq!(
        serde_querystring::from_str::<NewType>("-2500000"),
        Ok(NewType(-2_500_000))
    );
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
        serde_querystring::from_str::<Sample>(
            "neg_num=-2500&num=123&string=Hello&boolean=on&special_bool=false"
        ),
        Ok(s)
    );
}

#[test]
fn deserialize_repeated_keys() {
    let mut map = HashMap::new();
    map.insert("num".to_string(), -2501);
    assert_eq!(
        serde_querystring::from_str::<HashMap<String, i32>>(
            "num=-2500&num=-2503&num=-2502&num=-2501"
        ),
        Ok(map)
    );
}

#[test]
fn deserialize_plus_to_space() {
    let mut map = HashMap::new();
    map.insert("num".to_string(), "rum rum".to_string());
    map.insert("yum".to_string(), "ehem ehem".to_string());
    assert_eq!(
        serde_querystring::from_str::<HashMap<String, String>>("num=rum+rum&yum=ehem+ehem"),
        Ok(map)
    );
}

#[test]
fn deserialize_percentage_decoding() {
    let mut map = HashMap::new();
    map.insert("baba".to_string(), "بابابزرگ".to_string());
    map.insert("amoo".to_string(), "عمو نوروز".to_string());
    assert_eq!(
        serde_querystring::from_str::<HashMap<String, String>>(
            "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
            amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2"
        ),
        Ok(map)
    );
}

#[test]
fn deserialize_percentage_decoding_raw() {
    assert_eq!(
        serde_querystring::from_str::<String>("%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"),
        Ok("بابابزرگ".to_string())
    );
}

// It should be here, but it is not to keep the same behaviour as serde_urlencoded
// #[test]
// fn deserialize_percentage_invalid() {
//     assert!(serde_querystring::from_str::<String>("%00%01%11").is_err());
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
        serde_querystring::from_str::<Human>("favs=1,2,3&abblities=eats,reads,cries&age=69"),
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
        serde_querystring::from_str::<Human>(
            "child[age]=12&child[reads]=on&name=Regina+Phalange&book[pages]=300&book[finished]=true\
            &child[book][pages]=1000&child[book][finished]=off"
        ),
        human
    );

    assert_eq!(
        serde_querystring::from_str::<Human>(
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

    let deserialized = serde_querystring::from_str(
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

    assert_eq!(
        serde_querystring::from_str("weight[1,1,1]=1200&weight[2,2,2]=2400"),
        res
    );

    // This should also work as we ignore everything after the last ,
    assert_eq!(
        serde_querystring::from_str("weight[1,1,1,1]=1200&weight[2,2,2,2]=2400"),
        res
    );

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

    assert_eq!(
        serde_querystring::from_str("weight[1a,1a,1a]=big&weight[2a,2a,2a]=small"),
        res
    );

    // This should also work as we ignore everything after the last ,
    assert_eq!(
        serde_querystring::from_str("weight[1a,1a,1a,1a]=big&weight[2a,2a,2a,2a]=small"),
        res
    );
}

#[test]
fn deserialize_invalid_recursion() {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct A {
        a: BTreeMap<i32, A>,
    };

    // The form of a[2]=a[2]=4 should not be allowed, but it is for now
    let mut s = String::new();
    for _ in 0..1000 {
        s.push_str("a[2]=");
    }
    s.push('1');

    assert!(serde_querystring::from_str::<A>(&s).is_err());

    #[derive(Debug, Clone, Deserialize)]
    struct A2(Vec<A2>);

    assert!(serde_querystring::from_str::<A2>("1,1,1").is_err())
}

#[test]
fn deserialize_invalid_recursion_key() {
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct B {
        a: Box<Option<B>>,
    };

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
    struct A {
        a: B,
    };

    let mut s = "a".to_string();

    for _ in 0..128 {
        s.push_str("[a]");
    }

    s.push_str("=2");

    assert!(serde_querystring::from_str::<A>(&s).is_err());
}

#[test]
fn deserialize_number_overflow() {
    let max_u32_plus_1 = (u32::max_value() as i64) + 1;
    assert!(serde_querystring::from_str::<u32>(&max_u32_plus_1.to_string()).is_err());

    let max_i32_neg = -(max_u32_plus_1 / 2);
    assert!(serde_querystring::from_str::<u32>(&max_i32_neg.to_string()).is_err());

    assert!(serde_querystring::from_str::<u64>("18446744073709551616").is_err());

    assert!(serde_querystring::from_str::<i64>("-9223372036854775809").is_err());

    assert_eq!(
        serde_querystring::from_str::<f64>("18446744073709551616"),
        Ok(18446744073709551616_f64)
    );
    assert_eq!(
        serde_querystring::from_str::<f64>("-18446744073709551616"),
        Ok(-18446744073709551616_f64)
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

    assert_eq!(serde_querystring::from_str::<Side>("God"), Ok(Side::God));

    let mut map = HashMap::new();
    map.insert(Side::God, "winner");
    map.insert(Side::Right, "looser");
    assert_eq!(
        serde_querystring::from_str::<HashMap<Side, &str>>("God=winner&Right=looser"),
        Ok(map)
    );

    // as value
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        serde_querystring::from_str::<A>("looser=Left&winner=God"),
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
        serde_querystring::from_str::<B>(
            "sides=God,Left,Right&result[God]=10&result[Right]=-1&result[Left]=-1"
        ),
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

    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    struct A {
        last_event: Event,
    }

    assert_eq!(
        serde_querystring::from_str::<A>("last_event=PageLoad"),
        Ok(A {
            last_event: PageLoad
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("last_event[KeyPress]=2"),
        Ok(A {
            last_event: KeyPress('2')
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("last_event[Paste]=asd"),
        Ok(A {
            last_event: Paste("asd".to_string())
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("last_event[Missed]=1200,2400"),
        Ok(A {
            last_event: Missed(1200, 2400)
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("last_event[Click][x]=5500&last_event[Click][y]=6900"),
        Ok(A {
            last_event: Click { x: 5500, y: 6900 }
        })
    );
}

#[test]
fn deserialize_enum_in_seq() {
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

    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    struct A {
        events: Vec<Event>,
    }

    // unit enums
    assert_eq!(
        serde_querystring::from_str::<A>("events=PageLoad,PageUnload"),
        Ok(A {
            events: vec![PageLoad, PageUnload]
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("events[]=PageLoad&events[]=PageUnload"),
        Ok(A {
            events: vec![PageLoad, PageUnload]
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>(
            "events[][Paste]=asd&events[][Missed]=1200,2400\
            &events[group_1][Click][x]=5500&events[group_1][Click][y]=6900"
        ),
        Ok(A {
            events: vec![
                Paste("asd".to_string()),
                Missed(1200, 2400),
                Click { x: 5500, y: 6900 },
            ]
        })
    );
    assert_eq!(
        serde_querystring::from_str::<A>("events[][Missed]=1200,2400&events[][Paste]=asd"),
        Ok(A {
            events: vec![Missed(1200, 2400), Paste("asd".to_string())]
        })
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
        serde_querystring::from_str::<UvRate>("nums[]=1&nums[]=3&nums[]=1337&average=447"),
        Ok(UvRate {
            nums: vec![1, 3, 1337],
            average: 447
        })
    );

    assert_eq!(
        serde_querystring::from_str::<UvRate>("nums[0]=1&nums[1]=2&nums[2]=1337&average=300"),
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
        serde_querystring::from_str::<Country>(
            "sun[nums][]=1&sun[nums][]=3&sun[nums][]=1337&sun[average]=447"
        ),
        Ok(Country {
            sun: UvRate {
                nums: vec![1, 3, 1337],
                average: 447
            }
        })
    );

    assert_eq!(
        serde_querystring::from_str::<Country>(
            "sun[nums][0]=1&sun[nums][1]=2&sun[nums][2]=1337&sun[average]=300"
        ),
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

    assert_eq!(
        serde_querystring::from_str::<Weather>("Rainy[1]=h&Rainy[1]=v"),
        Ok(Weather::Rainy('h', 'v'))
    );
    assert_eq!(
        serde_querystring::from_str::<Weather>("Rainy=h,v"),
        Ok(Weather::Rainy('h', 'v'))
    );

    assert_eq!(
        serde_querystring::from_str::<Weather>("Sunny[uv]=300&Sunny[tempt]=3500"),
        Ok(Weather::Sunny {
            uv: 300,
            tempt: 3500
        })
    );

    assert_eq!(
        serde_querystring::from_str::<Weather>("Cold"),
        Ok(Weather::Cold)
    );

    assert_eq!(
        serde_querystring::from_str::<Weather>("Hot=10"),
        Ok(Weather::Hot(10))
    );

    #[derive(Debug, Deserialize, PartialEq)]
    struct City {
        history: Vec<Weather>,
    }

    assert_eq!(
        serde_querystring::from_str::<City>(
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
