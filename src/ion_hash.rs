use crate::ion_hash_encoder::encode_value;
use crate::IonValue;
use digest::Digest;
use sha2::Sha256;
use std::cmp::{Ordering, PartialEq};
use std::marker::PhantomData;

/// Ion Hash implementation. Once the hasher is initialized you can add new values to it
/// and it will perform the dot operation internally. Once you added everything you want
/// to add just call `get()` and it will provide you with a &[u8] slice containing the
/// hash.
///
/// You can use the method digest if you want to hash only one IonValue.
///
/// ```rust,no_run
/// use sha2::Sha256;
/// use ion_binary_rs::{IonHash, IonValue};
/// use std::collections::HashMap;
///
/// let mut ion_struct = HashMap::new();
///
/// ion_struct.insert("Model".to_string(), IonValue::String("CLK 350".to_string()));
/// ion_struct.insert("Type".to_string(), IonValue::String("Sedan".to_string()));
/// ion_struct.insert("Color".to_string(), IonValue::String("White".to_string()));
/// ion_struct.insert(
///     "VIN".to_string(),
///     IonValue::String("1C4RJFAG0FC625797".to_string()),
/// );
/// ion_struct.insert("Make".to_string(), IonValue::String("Mercedes".to_string()));
/// ion_struct.insert("Year".to_string(), IonValue::Integer(2019));
///
/// let ion_value = IonValue::Struct(ion_struct);
///
/// let hash = IonHash::digest::<Sha256>(&ion_value);
///
/// println!("{:X?}", hash);
/// ```
///
#[derive(Debug)]
pub struct IonHash<D: Digest = Sha256> {
    buffer: Vec<u8>,
    hasher_type: PhantomData<D>,
}

impl<D: Digest> IonHash<D> {
    /// Hashes the bytes and perform a dot operation with
    /// current version of the IonHash hash.
    pub fn add_bytes(&mut self, value: &[u8]) {
        let value = IonHash::from_bytes::<D>(value);

        self.dot(value);
    }

    /// Assumes that the bytes are already hashed and performs
    /// the dot operation with current version of the IonHash
    /// hash.
    pub fn add_hashed_bytes(&mut self, value: &[u8]) {
        let value = IonHash::from_hashes_bytes::<D>(value);

        self.dot(value);
    }

    /// Serializes and hashes the Ion Value and performs
    /// the dot operation with current version of the IonHash
    /// hash.
    pub fn add_ion_value(&mut self, value: &IonValue) {
        let buffer = encode_value::<D>(value);

        let value = IonHash::from_bytes::<D>(&buffer);

        self.dot(value);
    }

    /// performs the dot operation with current version of the
    /// IonHash hash.
    pub fn dot(&mut self, value: IonHash<D>) -> &mut Self {
        if value.buffer.is_empty() {
            return self;
        }

        if self.buffer.is_empty() {
            self.buffer = value.buffer;
            return self;
        }

        let mut buffer: Vec<u8> = vec![];

        if *self < value {
            buffer.extend(self.get());
            buffer.extend(value.get());
        } else {
            buffer.extend(value.get());
            buffer.extend(self.get());
        }

        self.buffer = D::digest(&buffer).to_vec();

        self
    }

    /// Gets the current hash. Useful for when you need to
    /// extract the final result after several operations.
    pub fn get(&self) -> &[u8] {
        &self.buffer
    }
}

impl IonHash {
    /// Creates an empty Ion Hash with the default hasher: Sha256
    pub fn new() -> IonHash {
        IonHash {
            buffer: vec![],
            hasher_type: PhantomData,
        }
    }

    /// Creates a hasher with some starting bytes which will
    /// be first hashed
    pub fn from_bytes<D: Digest>(buf: &[u8]) -> IonHash<D> {
        let hased_bytes = D::digest(buf);
        IonHash::from_hashes_bytes(&hased_bytes)
    }

    /// Creates a hasher with some starting hash
    pub fn from_hashes_bytes<D: Digest>(buf: &[u8]) -> IonHash<D> {
        IonHash {
            buffer: buf.to_vec(),
            hasher_type: PhantomData,
        }
    }

    /// Creates a hasher with some starting Ion Value which will
    /// be first serialized and hashed
    pub fn from_ion_value<D: Digest>(value: &IonValue) -> IonHash<D> {
        let mut hash = IonHash::with_hasher::<D>();

        hash.add_ion_value(value);

        hash
    }

    /// Creates an empty hasher using the provided hasher
    pub fn with_hasher<D: Digest>() -> IonHash<D> {
        IonHash {
            buffer: vec![],
            hasher_type: PhantomData,
        }
    }

    /// Shorthand method for hashing an Ion Value in one step.
    pub fn digest<D: Digest>(value: &IonValue) -> Vec<u8> {
        IonHash::from_ion_value::<D>(value).get().to_vec()
    }

    /// Shorthand method for hashing an Ion Value in one step.
    /// It uses the default hasher: Sha256
    pub fn default_digest(value: &IonValue) -> Vec<u8> {
        IonHash::from_ion_value::<Sha256>(value).get().to_vec()
    }
}

impl Default for IonHash {
    fn default() -> IonHash<Sha256> {
        IonHash::with_hasher::<Sha256>()
    }
}

impl<D: Digest> PartialEq for IonHash<D> {
    fn eq(&self, _: &IonHash<D>) -> bool {
        self.buffer == self.get()
    }
}

impl<D: Digest> PartialOrd for IonHash<D> {
    fn partial_cmp(&self, value: &IonHash<D>) -> Option<Ordering> {
        self.buffer
            .iter()
            .rev()
            .map(|byte| *byte as i8)
            .partial_cmp(value.get().iter().rev().map(|byte| *byte as i8))
    }
}
