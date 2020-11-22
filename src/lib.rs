//! # Serde-querystring
//! serde-querystring is a deserializer for query strings.
//!
//! Query strings are often used to send data to a web server as part of a http request and
//! they usually consist of a sequence of key/value pairs.
//! While they are commonly used, they don't have a strong standard defining how they should be parsed,
//! for anything more than just simple key/value pairs, so it is important to define how we are going to
//! parse them, and what is considered an expected behaviour and what can be a bug.
//! Serde-querystring tries to use the commonly used form `key[subkey]=value` pattern to provide
//! a way to define different `Rust` data structures.
//!
//! ## Before we begin
//! Formats used in de/serialization are either self-describing or non-self-describing which roughly
//! means either you can imagine the backing data structure only by looking at the serialized format or not.
//!
//! For example when looking at a `json` file, whenever you see some data in `{...}` form you can say it's a
//! map, same goes for `[...]` for a sequence/list. But for formats like query strings, by looking at
//! `key[1]=value&key[2]=value` you can't decide if key is a map with 1 and 2 as it's own keys,
//! or if it's a sequence with 2 elements. So it's the final data structure that defines how we
//! should look at the data.
//!
//! So instead of defining the format itself, we define how we can represent a data structure using the format.
//!
//! ## Keys
//! For every pair in a query string, there has to be a key(or key sequence) on the left hand side of `=` sign.
//! If the key is of a simple form `key` we consider it a group name for its value, if it has subkeys in the form
//! `key[subkey]` we read it as a group named `key` with some value that itself has a group called `subkey`.
//! So the terms `group name` and `key` are interchangeable here.
//!
//! ## Value
//! Values are the leaves of tree, meaning by looking at them we can instantly say what they are and
//! we don't need to go any deeper to undestand their meaning, in other words, they can't have any sub value or
//! sub keys(with one exception).
//!
//! | Types | Example valid representations |
//! |-------|------------------------------|
//! | `u64` `u32` `u16` `u8`        | `210` |
//! | `i64` `i32` `i16` `i8`        | `210` `-210` |
//! | `f64` `f32`                   | `1337`  `-1337`  `1337.4`  `-1337.4` `1.4E5` `1.2e-4` `1.9e+4` |
//! | `str`                         | `Hello` `World` |
//! | `String`                      | `Hello+World` `Hello%25World` `Hello` |
//! | `bool`                        | `on` `true` `1` for true and `off` `false` `0` for false |
//! | `enum` unit variants          | `Cold` `Dark` |
//! | `Option`(*)                   | `123` `Hello` depending on inner value or ` ` for None |
//! | `Vec` or `tuple` of values(*) | `210,340,450` |
//! | new type `struct`             | `123` `Cold,Warm` depending on inner value |
//!
//! * It is only considered a value if it consist of other values.
//!
//! ## Map/Struct
//! A query string starts either by a `map` or a `struct` at its root, these two are represented in the same
//! way and are considered the same kind of entity except when dealing with repeated keys(Described later).
//! To represent them, we start from the root's fields and consider every one of them a key, if the value
//! associated with that group/key also needs a sub group itself, we consider that group name as a subkey of main key.
//!
//! ### Example
//!
//! ```ignore
//! struct Home{
//!     lat: f64,
//!     long: f64
//! }
//! ```
//! Will be represented as: `lat=1.5&long=3.5`
//!
//! ```ignore
//! struct Area{
//!     gym: Home,
//!     police: Home
//! }
//! ```
//! Will be represented as: `gym[lat]=1.5&gym[long]=3.5&police[lat]=1.5&police[long]=3.5`
//!
//! ```ignore
//! type City = HashMap<String, Home>;
//! ```
//! Will be also represented as: `gym[lat]=1.5&gym[long]=3.5&police[lat]=1.5&police[long]=3.5`
//!
//! * Ordering of the fields does not matter, they can be combined in any possible order and it shouldn't
//! effect the result, unless there are repeated keys.
//! * For structs it's an error to repeat the same key or subkey more than once, but for maps the values are
//! overwritable.
//!
//! Validity of example cases
//!
//! | Case                                              | `City` | `Area` |
//! |---------------------------------------------------|--------|--------|
//! | `gym[lat]=1.5&gym[long]=3.5`                      |-       |X       |
//! | `...&police[lat]=1.5&police[long]=3.5`            |X       |X       |
//! | `gym[lat]=1.5&police[long]=3.5`                   |-       |-       |
//! | `...&gym[long]=1.5&police[lat]=3.5`               |X       |X       |
//! | `...&gym[long]=1.5&police[lat]=3.5`               |-(**)   |-(**)   |
//!
//! * (*)`X` means valid `-` means invalid `...` means continue from above
//! * (**) More on that on Repeated keys
//!
//! ## Sequences/Vectors/Tuples/Lists
//! Sequences are defined as a finite ordered set of groups, the group names in a sequence can be either empty
//! as in `key[]`, a string/name as in `key[first]` or a number as in `key[1]`. Here's what each kind of group
//! name can be used for:
//! - Empty: They can be used to represent some unspecified index for a value(as defined in `Value` section).
//! it can't have subkeys unless there are only one subkey(and subkey of subkey..) possible for it.
//! - String: They can be used to represent an unspecified index, and can also have subkeys.
//! - Number: Same as String but will also define the order.
//! * Keys in a sequence are read as the same order as they are defined, unspecified here means not defining a
//! specific order for a key and depending on the default order.
//! * In case of combining ordered and unordered, all unordered elements will come before ordered ones.
//! * In case of repeating a key, we only consider the ***last group*** defined. More on that on
//! "Repeated keys" section.
//!
//! ### Example
//! For a struct with only one field `a` of type `Vec<T>` for some type `T`
//!
//! | Value               | Example valid representation                      |
//! |---------------------|---------------------------------------------------|
//! | `[1,2]`             | `a[]=1&a[]=2`                                     |
//! | `[1,2]`             | `a[g2]=1&a[g1]=2`                                 |
//! | `[2]`               | `a[group]=1&a[group]=2`                           |
//! | `[2,1]`             | `a[2]=1&a[1]=2`                                   |
//! | `[3,2,1]`           | `a[2]=1&a[1]=2&a[]=3`                             |
//! | `[{X:1,Y:2}]` as map| `a[group][X]=1&a[group][Y]=2`                     |
//!
//! ## Enums
//! Enums in general can be represented the way as a map
//!
//! ### Example
//! For a struct defined as below
//! ```no_run
//! struct Game{
//!     last: Event
//! }
//!
//! // modified from rust by example book
//! enum Event {
//!     PageLoad,
//!     KeyPress(char),
//!     Paste(String),
//!     Click { x: i64, y: i64 },
//!     Missed(i32, i32),
//! }
//! ```
//! Here's how we represent different kinds of `Event`:
//!
//! | Value               | Example valid representation                      |
//! |---------------------|---------------------------------------------------|
//! |`PageLoad`           |`last=PageLoad` is valid as unit variants are value|
//! |`PageLoad`           |`last[PageLoad]=` is valid as value is unit        |
//! |`KeyPress('W')`      |`last[KeyPress]=W`                                 |
//! |`Paste("Hello")`     |`last[Paste]=Hello`                                |
//! |`Click{x:400,y:640}` |`last[Click][x]=400&last[Click][y]=640`            |
//! |`Missed(200,400)`    |`last[Missed]=200,400` list of values is a value   |
//! |`Missed(200,400)`    |`last[Missed][]=200&last[Missed][]=400`            |
//! |`Missed(200,400)`    |`last[Missed][1]=200&last[Missed][2]=400`          |
//!
//! ## Repeated keys
//! Though the keys in a map or list are overwritable, repeating keys assosiated with a struct is an error,
//! and as we consider keys, group names, the data structure in that level decides if it allows repeating or not.
//! Ex. for `map` of `struct`s it is not allowed, for `struct` with `map` fields it is allowed, for `struct` of
//! `map` fields containing `struct` as values it is again not allowed.
//!
//! Being overwritable for keys with subkeys(ex. Enums or Lists) creates a situation where we can't decide
//! what the final value should be, consider the example from `Enum` section, and the following query string:
//!
//! `last[Click][x]=400&last[Missed][]=200&last[Missed][]=400&last[Click][y]=640`
//!
//! In this kind of situations, to keep consistency with other kinds of keys, we only consider the
//! `last defined group`, so in the above example although the last key belongs to `Click` group name, we only
//! consider `Missed` group as it is defined after `Click`.
//!
//! #### Exception
//!
//! One exception is when there are enums in a map or sequence, and one of the enums are defined as a
//! value. In that case we visit the values first and we ignore the subkeys. So the following query strings
//! are all considered `PageLoad`.
//!
//! `last=PageLoad&last[KeyPress]=C`
//!
//! `last[KeyPress]=C&last=PageLoad`
//!
//! `last=PageUnload&last[KeyPress]=C&last=PageLoad`
//!
//! To overcome this, you can define all enum variants at the sublevel, ex:
//!
//! `last[PageLoad]=&last[KeyPress]=C`
//!

mod de;
mod error;

pub use de::{from_bytes, from_str};
pub use error::Error;
