//! Contains Value and ValueRef structs and its conversion traits.
//!
//! # Examples
//!
//! ```
//! ```

#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde;

use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::iter::FromIterator;
use std::ops::Index;
use std::str::Utf8Error;

use num_traits::NumCast;

pub mod decode;
pub mod encode;

#[cfg(feature = "with-serde")]
pub mod ext;

#[derive(Copy, Clone, Debug, PartialEq)]
enum IntPriv {
    /// Always non-less than zero.
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
}

/// Name of Serde newtype struct to Represent Msgpack's Ext
/// Msgpack Ext: Ext(tag, binary)
/// Serde data model: _ExtStruct((tag, binary))
/// Example Serde impl for custom type:
///
/// ```ignore
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// #[serde(rename = "_ExtStruct")]
/// struct ExtStruct((i8, serde_bytes::ByteBuf));
///
/// test_round(ExtStruct((2, serde_bytes::ByteBuf::from(vec![5]))),
///            Value::Ext(2, vec![5]));
/// ```
pub const MSGPACK_EXT_STRUCT_NAME: &str = "_ExtStruct";

/// Represents a MessagePack integer, whether signed or unsigned.
///
/// A `Value` or `ValueRef` that contains integer can be constructed using `From` trait.
#[derive(Copy, Clone, PartialEq)]
pub struct Integer {
    n: IntPriv,
}

impl Integer {
    /// Returns `true` if the integer can be represented as `i64`.
    #[inline]
    pub fn is_i64(&self) -> bool {
        match self.n {
            IntPriv::PosInt(n) => n <= std::i64::MAX as u64,
            IntPriv::NegInt(..) => true,
        }
    }

    /// Returns `true` if the integer can be represented as `u64`.
    #[inline]
    pub fn is_u64(&self) -> bool {
        match self.n {
            IntPriv::PosInt(..) => true,
            IntPriv::NegInt(..) => false,
        }
    }

    /// Returns the integer represented as `i64` if possible, or else `None`.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self.n {
            IntPriv::PosInt(n) => NumCast::from(n),
            IntPriv::NegInt(n) => Some(n),
        }
    }

    /// Returns the integer represented as `u64` if possible, or else `None`.
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self.n {
            IntPriv::PosInt(n) => Some(n),
            IntPriv::NegInt(n) => NumCast::from(n),
        }
    }

    /// Returns the integer represented as `f64` if possible, or else `None`.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self.n {
            IntPriv::PosInt(n) => NumCast::from(n),
            IntPriv::NegInt(n) => NumCast::from(n),
        }
    }
}

impl Debug for Integer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(&self.n, fmt)
    }
}

impl Display for Integer {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.n {
            IntPriv::PosInt(v) => Display::fmt(&v, fmt),
            IntPriv::NegInt(v) => Display::fmt(&v, fmt),
        }
    }
}

impl From<u8> for Integer {
    #[inline]
    fn from(n: u8) -> Self {
        Integer { n: IntPriv::PosInt(n as u64) }
    }
}

impl From<u16> for Integer {
    #[inline]
    fn from(n: u16) -> Self {
        Integer { n: IntPriv::PosInt(n as u64) }
    }
}

impl From<u32> for Integer {
    #[inline]
    fn from(n: u32) -> Self {
        Integer { n: IntPriv::PosInt(n as u64) }
    }
}

impl From<u64> for Integer {
    #[inline]
    fn from(n: u64) -> Self {
        Integer { n: IntPriv::PosInt(n as u64) }
    }
}

impl From<usize> for Integer {
    #[inline]
    fn from(n: usize) -> Self {
        Integer { n: IntPriv::PosInt(n as u64) }
    }
}

impl From<i8> for Integer {
    #[inline]
    fn from(n: i8) -> Self {
        if n < 0 {
            Integer { n: IntPriv::NegInt(n as i64) }
        } else {
            Integer { n: IntPriv::PosInt(n as u64) }
        }
    }
}

impl From<i16> for Integer {
    #[inline]
    fn from(n: i16) -> Self {
        if n < 0 {
            Integer { n: IntPriv::NegInt(n as i64) }
        } else {
            Integer { n: IntPriv::PosInt(n as u64) }
        }
    }
}

impl From<i32> for Integer {
    #[inline]
    fn from(n: i32) -> Self {
        if n < 0 {
            Integer { n: IntPriv::NegInt(n as i64) }
        } else {
            Integer { n: IntPriv::PosInt(n as u64) }
        }
    }
}

impl From<i64> for Integer {
    #[inline]
    fn from(n: i64) -> Self {
        if n < 0 {
            Integer { n: IntPriv::NegInt(n as i64) }
        } else {
            Integer { n: IntPriv::PosInt(n as u64) }
        }
    }
}

impl From<isize> for Integer {
    #[inline]
    fn from(n: isize) -> Self {
        if n < 0 {
            Integer { n: IntPriv::NegInt(n as i64) }
        } else {
            Integer { n: IntPriv::PosInt(n as u64) }
        }
    }
}

/// Represents any valid MessagePack value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    // /// Integer represents an integer.
    // ///
    // /// A value of an `Integer` object is limited from `-(2^63)` upto `(2^64)-1`.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use rbmpv::Value;
    // ///
    // /// assert_eq!(42, Value::from(42).as_i64().unwrap());
    // /// ```
    // Integer(Integer),
    I64(i64),
    // Integer(Integer),
    U64(u64),

    /// A 32-bit floating point number.
    F32(f32),
    /// A 64-bit floating point number.
    F64(f64),
    /// String extending Raw type represents a UTF-8 string.
    ///
    /// # Note
    ///
    /// String objects may contain invalid byte sequence and the behavior of a deserializer depends
    /// on the actual implementation when it received invalid byte sequence. Deserializers should
    /// provide functionality to get the original byte array so that applications can decide how to
    /// handle the object
    String(Vec<u8>),
    /// Binary extending Raw type represents a byte array.
    Binary(Vec<u8>),
    /// Array represents a sequence of objects.
    Array(Vec<Value>),
    /// Map represents key-value pairs of objects.
    Map(Vec<(Value, Value)>),
    /// Extended implements Extension interface: represents a tuple of type information and a byte
    /// array where type information is an integer whose meaning is defined by applications.
    Ext(i8, Vec<u8>),
}

impl Value {
    /// Converts the current owned Value to a ValueRef.
    ///
    /// # Panics
    ///
    /// Panics in unable to allocate memory to keep all internal structures and buffers.
    ///
    /// # Examples
    /// ```
    /// use rbmpv::{Value, ValueRef};
    ///
    /// let val = Value::Array(vec![
    ///     Value::Nil,
    ///     Value::from(42),
    ///     Value::Array(vec![
    ///         Value::String("le message".into())
    ///     ])
    /// ]);
    ///
    /// let expected = ValueRef::Array(vec![
    ///    ValueRef::Nil,
    ///    ValueRef::from(42),
    ///    ValueRef::Array(vec![
    ///        ValueRef::from("le message"),
    ///    ])
    /// ]);
    ///
    /// assert_eq!(expected, val.as_ref());
    /// ```
    pub fn as_ref(&self) -> ValueRef<'_> {
        match *self {
            Value::Nil => ValueRef::Nil,
            Value::Boolean(val) => ValueRef::Boolean(val),
            Value::I64(val) => ValueRef::I64(val),
            Value::U64(val) => ValueRef::U64(val),
            Value::F32(val) => ValueRef::F32(val),
            Value::F64(val) => ValueRef::F64(val),
            Value::String(ref val) => ValueRef::String(val.as_ref()),
            Value::Binary(ref val) => ValueRef::Binary(val.as_slice()),
            Value::Array(ref val) => {
                ValueRef::Array(val.iter().map(|v| v.as_ref()).collect())
            }
            Value::Map(ref val) => {
                ValueRef::Map(val.iter().map(|&(ref k, ref v)| (k.as_ref(), v.as_ref())).collect())
            }
            Value::Ext(ty, ref buf) => ValueRef::Ext(ty, buf.as_slice()),
        }
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::Nil.is_nil());
    /// ```
    #[inline]
    pub fn is_nil(&self) -> bool {
        if let Value::Nil = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::Boolean(true).is_bool());
    ///
    /// assert!(!Value::Nil.is_bool());
    /// ```
    #[inline]
    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Returns true if the `Value` is convertible to an i64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::from(42).is_i64());
    ///
    /// assert!(!Value::from(42.0).is_i64());
    /// ```
    #[inline]
    pub fn is_i64(&self) -> bool {
        if let Value::I64(ref v) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if the `Value` is convertible to an u64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::from(42).is_u64());
    ///
    /// assert!(!Value::F32(42.0).is_u64());
    /// assert!(!Value::F64(42.0).is_u64());
    /// ```
    #[inline]
    pub fn is_u64(&self) -> bool {
        if let Value::U64(ref v) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if (and only if) the `Value` is a f32. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::F32(42.0).is_f32());
    ///
    /// assert!(!Value::from(42).is_f32());
    /// assert!(!Value::F64(42.0).is_f32());
    /// ```
    #[inline]
    pub fn is_f32(&self) -> bool {
        if let Value::F32(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if (and only if) the `Value` is a f64. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::F64(42.0).is_f64());
    ///
    /// assert!(!Value::from(42).is_f64());
    /// assert!(!Value::F32(42.0).is_f64());
    /// ```
    #[inline]
    pub fn is_f64(&self) -> bool {
        if let Value::F64(..) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::from(42).is_number());
    /// assert!(Value::F32(42.0).is_number());
    /// assert!(Value::F64(42.0).is_number());
    ///
    /// assert!(!Value::Nil.is_number());
    /// ```
    pub fn is_number(&self) -> bool {
        match *self {
            Value::I64(..) | Value::U64(..) | Value::F32(..) | Value::F64(..) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert!(Value::String("value".into()).is_str());
    ///
    /// assert!(!Value::Nil.is_str());
    /// ```
    #[inline]
    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// Returns true if the `Value` is a Binary. Returns false otherwise.
    #[inline]
    pub fn is_bin(&self) -> bool {
        self.as_slice().is_some()
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    #[inline]
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Returns true if the `Value` is a Map. Returns false otherwise.
    #[inline]
    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    /// Returns true if the `Value` is an Ext. Returns false otherwise.
    #[inline]
    pub fn is_ext(&self) -> bool {
        self.as_ext().is_some()
    }

    /// If the `Value` is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some(true), Value::Boolean(true).as_bool());
    ///
    /// assert_eq!(None, Value::Nil.as_bool());
    /// ```
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Boolean(val) = *self {
            Some(val)
        } else {
            None
        }
    }

    /// If the `Value` is an integer, return or cast it to a i64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some(42i64), Value::from(42).as_i64());
    ///
    /// assert_eq!(None, Value::F64(42.0).as_i64());
    /// ```
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(ref n) => Some(n.to_owned()),
            _ => None,
        }
    }

    /// If the `Value` is an integer, return or cast it to a u64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some(42u64), Value::from(42).as_u64());
    ///
    /// assert_eq!(None, Value::from(-42).as_u64());
    /// assert_eq!(None, Value::F64(42.0).as_u64());
    /// ```
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::U64(ref n) => Some(n.to_owned()),
            _ => None,
        }
    }

    /// If the `Value` is a number, return or cast it to a f64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some(42.0), Value::from(42).as_f64());
    /// assert_eq!(Some(42.0), Value::F32(42.0f32).as_f64());
    /// assert_eq!(Some(42.0), Value::F64(42.0f64).as_f64());
    ///
    /// assert_eq!(Some(2147483647.0), Value::from(i32::max_value() as i64).as_f64());
    ///
    /// assert_eq!(None, Value::Nil.as_f64());
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::I64(n) => Some(n as f64),
            Value::U64(n) => Some(n as f64),
            Value::F32(n) => Some(From::from(n)),
            Value::F64(n) => Some(n),
            _ => None,
        }
    }

    /// If the `Value` is a String, returns the associated str.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some("le message"), Value::String("le message".into()).as_str());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_str());
    /// ```
    #[inline]
    pub fn as_str(&self) -> Option<Cow<'_,str>> {
        if let Value::String(ref val) = *self {
            Some(String::from_utf8_lossy(val))
        } else {
            None
        }
    }

    /// If the `Value` is a Binary or a String, returns the associated slice.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some(&[1, 2, 3, 4, 5][..]), Value::Binary(vec![1, 2, 3, 4, 5]).as_slice());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_slice());
    /// ```
    pub fn as_slice(&self) -> Option<&[u8]> {
        if let Value::Binary(ref val) = *self {
            Some(val)
        } else if let Value::String(ref val) = *self {
            Some(val)
        } else {
            None
        }
    }

    /// If the `Value` is an Array, returns the associated vector.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// let val = Value::Array(vec![Value::Nil, Value::Boolean(true)]);
    ///
    /// assert_eq!(Some(&vec![Value::Nil, Value::Boolean(true)]), val.as_array());
    ///
    /// assert_eq!(None, Value::Nil.as_array());
    /// ```
    #[inline]
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(ref array) = *self {
            Some(&*array)
        } else {
            None
        }
    }

    /// If the `Value` is a Map, returns the associated vector of key-value tuples.
    /// Returns None otherwise.
    ///
    /// # Note
    ///
    /// MessagePack represents map as a vector of key-value tuples.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// let val = Value::Map(vec![(Value::Nil, Value::Boolean(true))]);
    ///
    /// assert_eq!(Some(&vec![(Value::Nil, Value::Boolean(true))]), val.as_map());
    ///
    /// assert_eq!(None, Value::Nil.as_map());
    /// ```
    #[inline]
    pub fn as_map(&self) -> Option<&Vec<(Value, Value)>> {
        if let Value::Map(ref map) = *self {
            Some(map)
        } else {
            None
        }
    }

    /// If the `Value` is an Ext, returns the associated tuple with a ty and slice.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::Value;
    ///
    /// assert_eq!(Some((42, &[1, 2, 3, 4, 5][..])), Value::Ext(42, vec![1, 2, 3, 4, 5]).as_ext());
    ///
    /// assert_eq!(None, Value::Boolean(true).as_ext());
    /// ```
    #[inline]
    pub fn as_ext(&self) -> Option<(i8, &[u8])> {
        if let Value::Ext(ty, ref buf) = *self {
            Some((ty, buf))
        } else {
            None
        }
    }
}

static NIL: Value = Value::Nil;
static NIL_REF: ValueRef<'static> = ValueRef::Nil;

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Value {
        self.as_array().and_then(|v| v.get(index)).unwrap_or(&NIL)
    }
}

impl Index<&str> for Value {
    type Output = Value;
    fn index(&self, index: &str) -> &Value {
        if let Value::Map(ref map) = *self {
            if let Some(found) = map.iter().find(
                |(key, _val)| {
                    if let Value::String(ref strval) = *key {
                        if let Some(s) = key.as_str() {
                            if s == index { return true; }
                        }
                    }
                    return false;
                })
            {
                return &found.1;
            }
        }
        &NIL
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<u8> for Value {
    #[inline]
    fn from(v: u8) -> Self {
        Value::U64(From::from(v))
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(v: u16) -> Self {
        Value::U64(From::from(v))
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(v: u32) -> Self {
        Value::U64(From::from(v))
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(v: u64) -> Self {
        Value::U64(From::from(v))
    }
}

impl From<usize> for Value {
    #[inline]
    fn from(v: usize) -> Self {
        Value::U64(From::from(v as u64))
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(v: i8) -> Self {
        Value::I64(From::from(v))
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(v: i16) -> Self {
        Value::I64(From::from(v))
    }
}

impl From<i32> for Value {
    #[inline]
    fn from(v: i32) -> Self {
        Value::I64(From::from(v))
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(v: i64) -> Self {
        Value::I64(From::from(v))
    }
}

impl From<isize> for Value {
    #[inline]
    fn from(v: isize) -> Self {
        Value::I64(From::from(v as i64))
    }
}

impl From<f32> for Value {
    #[inline]
    fn from(v: f32) -> Self {
        Value::F32(v)
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(v: f64) -> Self {
        Value::F64(v)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(v: String) -> Self {
        Value::String(v.into_bytes())
    }
}

impl<'a> From<&'a str> for Value {
    #[inline]
    fn from(v: &str) -> Self {
        Value::String(v.as_bytes().to_vec())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    #[inline]
    fn from(v: Cow<'a, str>) -> Self {
        Value::String(v.as_ref().as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Value {
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        Value::Binary(v)
    }
}

impl<'a> From<&'a [u8]> for Value {
    #[inline]
    fn from(v: &[u8]) -> Self {
        Value::Binary(v.into())
    }
}

impl<'a> From<Cow<'a, [u8]>> for Value {
    #[inline]
    fn from(v: Cow<'a, [u8]>) -> Self {
        Value::Binary(v.into_owned())
    }
}

impl From<Vec<Value>> for Value {
    #[inline]
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl From<Vec<(Value, Value)>> for Value {
    #[inline]
    fn from(v: Vec<(Value, Value)>) -> Self {
        Value::Map(v)
    }
}

/// Note that an `Iterator<Item = u8>` will be collected into an
/// [`Array`](crate::Value::Array), rather than a
/// [`Binary`](crate::Value::Binary)
impl<V> FromIterator<V> for Value
    where V: Into<Value> {
    fn from_iter<I: IntoIterator<Item=V>>(iter: I) -> Self {
        let v: Vec<Value> = iter.into_iter().map(|v| v.into()).collect();
        Value::Array(v)
    }
}

impl TryFrom<Value> for u64 {
    type Error = Value;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        match val {
            Value::U64(n) => {
                Ok(n)
            }
            v => Err(v),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = Value;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        match val {
            Value::I64(n) => {
                Ok(n)
            }
            v => Err(v),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = Value;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        match val {
            Value::F32(n) => Ok(From::from(n)),
            Value::F64(n) => Ok(n),
            v => Err(v),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = Value;

    fn try_from(val: Value) -> Result<Self, Self::Error> {
        match val {
            Value::String(u) => {
                Ok(String::from_utf8(u).unwrap_or_default())
            }
            _ => Err(val)
        }
    }
}
// The following impl was left out intentionally, see
// https://github.com/3Hren/msgpack-rust/pull/228#discussion_r359513925
/*
impl TryFrom<Value> for (i8, Vec<u8>) {
  type Error = Value;

  fn try_from(val: Value) -> Result<Self, Self::Error> {
      match val {
        Value::Ext(i, v) => Ok((i, v)),
        v => Err(v),
      }
  }
}
*/

macro_rules! impl_try_from {
  ($t: ty, $p: ident) => {
    impl TryFrom<Value> for $t {
      type Error = Value;

      fn try_from(val: Value) -> Result<$t, Self::Error> {
        match val {
          Value::$p(v) => Ok(v),
          v => Err(v)
        }
      }
    }
  };
}

impl_try_from!(bool, Boolean);
impl_try_from!(Vec<Value>, Array);
impl_try_from!(Vec<(Value, Value)>, Map);
impl_try_from!(Vec<u8>, Binary);
impl_try_from!(f32, F32);
// impl_try_from!(Utf8String, String);

impl Display for Value {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Value::Nil => f.write_str("nil"),
            Value::Boolean(val) => Display::fmt(&val, f),
            Value::I64(ref val) => Display::fmt(&val, f),
            Value::U64(ref val) => Display::fmt(&val, f),
            Value::F32(val) => Display::fmt(&val, f),
            Value::F64(val) => Display::fmt(&val, f),
            Value::String(ref val) => Display::fmt(&self.as_str().unwrap_or_default(), f),
            Value::Binary(ref val) => Debug::fmt(val, f),
            Value::Array(ref vec) => {
                // TODO: This can be slower than naive implementation. Need benchmarks for more
                // information.
                let res = vec.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "[{}]", res)
            }
            Value::Map(ref vec) => {
                write!(f, "{{")?;

                match vec.iter().take(1).next() {
                    Some(&(ref k, ref v)) => {
                        write!(f, "{}: {}", k, v)?;
                    }
                    None => {
                        write!(f, "")?;
                    }
                }

                for &(ref k, ref v) in vec.iter().skip(1) {
                    write!(f, ", {}: {}", k, v)?;
                }

                write!(f, "}}")
            }
            Value::Ext(ty, ref data) => {
                write!(f, "[{}, {:?}]", ty, data)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueRef<'a> {
    /// Nil represents nil.
    Nil,
    /// Boolean represents true or false.
    Boolean(bool),
    /// Integer represents an integer.
    ///
    /// A value of an `Integer` object is limited from `-(2^63)` upto `(2^64)-1`.
    // Integer(Integer),
    I64(i64),
    U64(u64),
    /// A 32-bit floating point number.
    F32(f32),
    /// A 64-bit floating point number.
    F64(f64),
    /// String extending Raw type represents a UTF-8 string.
    String(&'a [u8]),
    /// Binary extending Raw type represents a byte array.
    Binary(&'a [u8]),
    /// Array represents a sequence of objects.
    Array(Vec<ValueRef<'a>>),
    /// Map represents key-value pairs of objects.
    Map(Vec<(ValueRef<'a>, ValueRef<'a>)>),
    /// Extended implements Extension interface: represents a tuple of type information and a byte
    /// array where type information is an integer whose meaning is defined by applications.
    Ext(i8, &'a [u8]),
}

impl<'a> ValueRef<'a> {
    /// Converts the current non-owning value to an owned Value.
    ///
    /// This is achieved by deep copying all underlying structures and borrowed buffers.
    ///
    /// # Panics
    ///
    /// Panics in unable to allocate memory to keep all internal structures and buffers.
    ///
    /// # Examples
    /// ```
    /// use rbmpv::{Value, ValueRef};
    ///
    /// let val = ValueRef::Array(vec![
    ///    ValueRef::Nil,
    ///    ValueRef::from(42),
    ///    ValueRef::Array(vec![
    ///        ValueRef::from("le message"),
    ///    ])
    /// ]);
    ///
    /// let expected = Value::Array(vec![
    ///     Value::Nil,
    ///     Value::from(42),
    ///     Value::Array(vec![
    ///         Value::String("le message".into())
    ///     ])
    /// ]);
    ///
    /// assert_eq!(expected, val.to_owned());
    /// ```
    pub fn to_owned(&self) -> Value {
        match *self {
            ValueRef::Nil => Value::Nil,
            ValueRef::Boolean(val) => Value::Boolean(val),
            ValueRef::I64(val) => Value::I64(val),
            ValueRef::U64(val) => Value::U64(val),
            ValueRef::F32(val) => Value::F32(val),
            ValueRef::F64(val) => Value::F64(val),
            ValueRef::String(val) => Value::String(val.into()),
            ValueRef::Binary(val) => Value::Binary(val.to_vec()),
            ValueRef::Array(ref val) => {
                Value::Array(val.iter().map(|v| v.to_owned()).collect())
            }
            ValueRef::Map(ref val) => {
                Value::Map(val.iter().map(|&(ref k, ref v)| (k.to_owned(), v.to_owned())).collect())
            }
            ValueRef::Ext(ty, buf) => Value::Ext(ty, buf.to_vec()),
        }
    }

    pub fn index(&self, index: usize) -> &ValueRef<'_> {
        self.as_array().and_then(|v| v.get(index)).unwrap_or(&NIL_REF)
    }

    /// If the `ValueRef` is an integer, return or cast it to a u64.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::ValueRef;
    ///
    /// assert_eq!(Some(42), ValueRef::from(42).as_u64());
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            ValueRef::U64( n) => Some(n),
            _ => None,
        }
    }

    /// If the `ValueRef` is an Array, returns the associated vector.
    /// Returns None otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rbmpv::ValueRef;
    ///
    /// let val = ValueRef::Array(vec![ValueRef::Nil, ValueRef::Boolean(true)]);
    ///
    /// assert_eq!(Some(&vec![ValueRef::Nil, ValueRef::Boolean(true)]), val.as_array());
    /// assert_eq!(None, ValueRef::Nil.as_array());
    /// ```
    pub fn as_array(&self) -> Option<&Vec<ValueRef<'_>>> {
        if let ValueRef::Array(ref array) = *self {
            Some(&*array)
        } else {
            None
        }
    }

    #[inline]
    pub fn into_array(self) -> Option<Vec<ValueRef<'a>>> {
        if let ValueRef::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<Cow<'_,str>> {
        if let ValueRef::String(val) = *self {
            Some(String::from_utf8_lossy(val))
        } else {
            None
        }
    }
}

impl<'a> From<u8> for ValueRef<'a> {
    #[inline]
    fn from(v: u8) -> Self {
        ValueRef::U64(From::from(v))
    }
}

impl<'a> From<u16> for ValueRef<'a> {
    #[inline]
    fn from(v: u16) -> Self {
        ValueRef::U64(From::from(v))
    }
}

impl<'a> From<u32> for ValueRef<'a> {
    #[inline]
    fn from(v: u32) -> Self {
        ValueRef::U64(From::from(v))
    }
}

impl<'a> From<u64> for ValueRef<'a> {
    #[inline]
    fn from(v: u64) -> Self {
        ValueRef::U64(From::from(v))
    }
}

impl<'a> From<usize> for ValueRef<'a> {
    #[inline]
    fn from(v: usize) -> Self {
        ValueRef::U64(v as u64)
    }
}

impl<'a> From<i8> for ValueRef<'a> {
    #[inline]
    fn from(v: i8) -> Self {
        ValueRef::I64(From::from(v))
    }
}

impl<'a> From<i16> for ValueRef<'a> {
    #[inline]
    fn from(v: i16) -> Self {
        ValueRef::I64(From::from(v))
    }
}

impl<'a> From<i32> for ValueRef<'a> {
    #[inline]
    fn from(v: i32) -> Self {
        ValueRef::I64(From::from(v))
    }
}

impl<'a> From<i64> for ValueRef<'a> {
    #[inline]
    fn from(v: i64) -> Self {
        ValueRef::I64(From::from(v))
    }
}

impl<'a> From<isize> for ValueRef<'a> {
    #[inline]
    fn from(v: isize) -> Self {
        ValueRef::I64(v as i64)
    }
}

impl<'a> From<f32> for ValueRef<'a> {
    #[inline]
    fn from(v: f32) -> Self {
        ValueRef::F32(v)
    }
}

impl<'a> From<f64> for ValueRef<'a> {
    #[inline]
    fn from(v: f64) -> Self {
        ValueRef::F64(v)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    #[inline]
    fn from(v: &'a str) -> Self {
        ValueRef::String(v.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for ValueRef<'a> {
    #[inline]
    fn from(v: &'a [u8]) -> Self {
        ValueRef::Binary(v)
    }
}

impl<'a> From<Vec<ValueRef<'a>>> for ValueRef<'a> {
    #[inline]
    fn from(v: Vec<ValueRef<'a>>) -> Self {
        ValueRef::Array(v)
    }
}

/// Note that an `Iterator<Item = u8>` will be collected into an
/// [`Array`](crate::Value::Array), rather than a
/// [`Binary`](crate::Value::Binary)
impl<'a, V> FromIterator<V> for ValueRef<'a>
    where V: Into<ValueRef<'a>> {
    fn from_iter<I: IntoIterator<Item=V>>(iter: I) -> Self {
        let v: Vec<ValueRef<'a>> = iter.into_iter().map(|v| v.into()).collect();
        ValueRef::Array(v)
    }
}

impl<'a> From<Vec<(ValueRef<'a>, ValueRef<'a>)>> for ValueRef<'a> {
    fn from(v: Vec<(ValueRef<'a>, ValueRef<'a>)>) -> Self {
        ValueRef::Map(v)
    }
}

impl<'a> TryFrom<ValueRef<'a>> for u64 {
    type Error = ValueRef<'a>;

    fn try_from(val: ValueRef<'a>) -> Result<Self, Self::Error> {
        match val {
            ValueRef::U64(n) => {
               Ok(n)
            }
            v => Err(v),
        }
    }
}

// The following impl was left out intentionally, see
// https://github.com/3Hren/msgpack-rust/pull/228#discussion_r359513925
/*
impl<'a> TryFrom<ValueRef<'a>> for (i8, &'a[u8]) {
  type Error = ValueRef<'a>;

  fn try_from(val: ValueRef<'a>) -> Result<Self, Self::Error> {
      match val {
        ValueRef::Ext(i, v) => Ok((i, v)),
        v => Err(v),
      }
  }
}
*/

macro_rules! impl_try_from_ref {
  ($t: ty, $p: ident) => {
    impl<'a> TryFrom<ValueRef<'a>> for $t {
      type Error = ValueRef<'a>;

      fn try_from(val: ValueRef<'a>) -> Result<$t, Self::Error> {
        match val {
          ValueRef::$p(v) => Ok(v),
          v => Err(v)
        }
      }
    }
  };
}

impl_try_from_ref!(bool, Boolean);
impl_try_from_ref!(Vec<ValueRef<'a>>, Array);
impl_try_from_ref!(Vec<(ValueRef<'a>, ValueRef<'a>)>, Map);
impl_try_from_ref!(&'a [u8], Binary);
impl_try_from_ref!(f32, F32);
// impl_try_from_ref!(Utf8StringRef<'a>, String);

impl<'a> Display for ValueRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            ValueRef::Nil => write!(f, "nil"),
            ValueRef::Boolean(val) => Display::fmt(&val, f),
            ValueRef::I64(ref val) => Display::fmt(&val, f),
            ValueRef::U64(ref val) => Display::fmt(&val, f),
            ValueRef::F32(ref val) => Display::fmt(&val, f),
            ValueRef::F64(ref val) => Display::fmt(&val, f),
            ValueRef::String(ref val) => Display::fmt(&String::from_utf8(val.to_vec()).unwrap_or_default(), f),
            ValueRef::Binary(ref val) => Debug::fmt(&String::from_utf8(val.to_vec()).unwrap_or_default(), f),
            ValueRef::Array(ref vec) => {
                let res = vec.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "[{}]", res)
            }
            ValueRef::Map(ref vec) => {
                write!(f, "{{")?;

                match vec.iter().take(1).next() {
                    Some(&(ref k, ref v)) => {
                        write!(f, "{}: {}", k, v)?;
                    }
                    None => {
                        write!(f, "")?;
                    }
                }

                for &(ref k, ref v) in vec.iter().skip(1) {
                    write!(f, ", {}: {}", k, v)?;
                }

                write!(f, "}}")
            }
            ValueRef::Ext(ty, ref data) => {
                write!(f, "[{}, {:?}]", ty, data)
            }
        }
    }
}
