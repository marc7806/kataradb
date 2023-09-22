// Object types
pub const OBJ_TYPE_STRING: u8 = 0b0000_0000;

// Object Encodings
pub const OBJ_ENCODING_RAW: u8 = 0b0000_0000;
pub const OBJ_ENCODING_INT: u8 = 0b0000_0001;
pub const OBJ_ENCODING_EMBSTR: u8 = 0b0000_1000;

pub fn get_string_encoding(value: &String) -> u8 {
    if value.parse::<i64>().is_ok() {
        return OBJ_ENCODING_INT;
    }

    // check whether length of string is less than 44 bytes
    if value.len() < 44 {
        return OBJ_ENCODING_EMBSTR;
    }

    return OBJ_ENCODING_RAW;
}

pub fn get_type(type_encoding: u8) -> u8 {
    return type_encoding & 0b1111_0000;
}
