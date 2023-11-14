pub trait UnitConversion {
    fn mebibytes(self) -> Self;
    fn kibibytes(self) -> Self;
}

impl<N: num_traits::CheckedMul + num_traits::FromPrimitive> UnitConversion for N {
    fn mebibytes(self) -> Self {
        self.checked_mul(&N::from_u32(1_048_576).unwrap()).unwrap()
    }
    fn kibibytes(self) -> Self {
        self.checked_mul(&N::from_u32(1_024).unwrap()).unwrap()
    }
}
