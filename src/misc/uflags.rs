#[derive(Copy, Clone)]
pub struct Flags16(pub u16);
impl Flags16 {
    pub fn from_u16(num: u16) -> Flags16 {
        Flags16(num)
    }
    pub fn into_u16(&self) -> u16 {
        self.0
    }
    pub fn check_flag(&self, index: usize) -> Option<bool> {
        if index > 15 {
            return None;
        }
        match self.getbit(index) {
            0u16 => return Some(true),
            1u16 => return Some(false),
            _ => return None, //shouldn't ever happen
        }
    }
    pub fn getbit(&self, index: usize) -> u16 {
        //Returns only 0 or 1
        (self.0 & (1 << index)) >> index
    }
    pub fn truncate_bits(&self, quantity: isize) -> Flags16 {
        //Quantity is subtracted by 1 as otherwise an input of 3 would lead to 4 bits being truncated, etc.
        if quantity - 1 > 15 {
            return Self::from_u16(0);
        }
        Self::from_u16(self.0 & !(((1 << (quantity - 1)) - 1) << (15 - (quantity - 1))))
    }
}

impl PartialEq for Flags16 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
