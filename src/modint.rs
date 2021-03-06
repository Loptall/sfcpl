pub use modint::*;

pub mod modint {
    use num_integer::Integer;
    use num_traits::identities::{One, Zero};
    use num_traits::{Num, Pow};
    use std::cmp::Ordering;
    use std::convert::TryInto;
    use std::num::NonZeroU32;
    use std::num::ParseIntError;
    use std::ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign,
    };

    use crate::combinatorics::factorial::Factoriable;

    /// n % m
    /// ただし答えが負になる場合は余分にmを足すことで一意な値を保証
    ///
    /// # Pani,c
    /// 異なるmod間での演算をattemptした時
    fn compensated_rem(n: i64, m: usize) -> i64 {
        match n % m as i64 {
            // あまりが非負ならそのまま
            r if r >= 0 => r,
            // あまりが負ならmodを足す
            r => r + m as i64,
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum Modulo {
        Static(NonZeroU32),
        Dynamic,
    }

    impl Modulo {
        pub fn get(&self) -> Option<u32> {
            match self {
                Modulo::Static(nz) => Some(nz.get()),
                Modulo::Dynamic => None,
            }
        }
    }

    /// `ModInt -> PrimiteveInt` への暗黙のキャストは行わない!
    /// (get関数を提供するのでそれ使ってどうぞ)
    ///
    /// `PrimitiveInt -> ModInt` は許可する
    #[derive(Debug, Clone, Copy)]
    pub struct ModInt {
        num: i64,
        _modulo: Modulo,
    }

    impl Into<usize> for ModInt {
        fn into(self) -> usize {
            self.get() as usize
        }
    }

    pub trait IntoModInt: Copy {
        fn to_mint<M: TryInto<u32> + Copy>(self, modulo: M) -> ModInt;
    }

    macro_rules! impl_into_mint {
    ($($t:ty),*) => {
        $(
            impl IntoModInt for $t {
                fn to_mint<M: TryInto<u32> + Copy>(self, modulo: M) -> ModInt {
                    ModInt::new(self, modulo)
                }
            }
        )*
    };
}

    impl_into_mint!(usize, u8, u16, u32, u64, isize, i8, i16, i32, i64);

    impl PartialEq for ModInt {
        fn eq(&self, other: &Self) -> bool {
            if !check_mod_eq(self, other).1 {
                panic!("cannot compare these values because they have different modulo number",)
            }
            self.get() == other.num
        }
    }

    // #[snippet("modint")]
    // impl Eq for ModInt {}

    impl PartialOrd for ModInt {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            if !check_mod_eq(self, other).1 {
                None
            } else {
                Some(self.get().cmp(&other.num))
            }
        }
    }

    // #[snippet("modint")]
    // impl Ord for ModInt {
    //     fn cmp(&self, other: &Self) -> Ordering {
    //         self.partial_cmp(other).unwrap()
    //     }
    // }

    fn check_mod_eq(a: &ModInt, b: &ModInt) -> (NonZeroU32, bool) {
        match (a._modulo, b._modulo) {
            (Modulo::Static(a), Modulo::Static(b)) => {
                if a == b {
                    (a, true)
                } else {
                    // safe becase 1 != 0, yeah
                    (unsafe { NonZeroU32::new_unchecked(1) }, false)
                }
            }
            (Modulo::Static(m), Modulo::Dynamic) | (Modulo::Dynamic, Modulo::Static(m)) => {
                (m, true)
            }
            (Modulo::Dynamic, Modulo::Dynamic) => (unsafe { NonZeroU32::new_unchecked(1) }, false),
        }
    }

    impl ModInt {
        /// always `_modulo > num >= 0 && _modulo >= 1`
        pub fn new<N: TryInto<i64>, M: TryInto<u32> + Copy>(n: N, m: M) -> Self {
            let m =
                NonZeroU32::new(m.try_into().ok().expect("modulo number may be wrong")).unwrap();
            let r = n
                .try_into()
                .ok()
                .expect("modulo number maybe over i64 range");
            let num = compensated_rem(r, m.get() as usize);
            Self {
                num,
                _modulo: Modulo::Static(m),
            }
        }

        /// get inner value
        pub fn get(&self) -> i64 {
            self.num
        }

        /// mod of modint
        ///
        /// # Pani,c
        /// if variant is Modulo::Dynamic
        pub fn get_mod(&self) -> usize {
            self._modulo.get().unwrap() as usize
        }

        /// return the power of self with mod, using binary powering method
        /// cannot use of Dynamic type mod Self
        fn pow_mod(&self, mut exp: usize) -> Self {
            let mut res = 1;
            let mut base = self.get() as usize;
            let m = self.get_mod();
            while exp > 0 {
                if exp & 1 != 0 {
                    res *= base;
                    res %= m;
                }
                base *= base;
                base %= m;
                exp >>= 1;
            }

            Self::new(res, self.get_mod())
        }

        /// `a / b == a * b^(-1)` となる `b^(-1)` を求める
        pub fn inv(&self) -> i64 {
            // let mut a = self.get();
            // let m = self.get_mod() as i64;
            // let mut b = self.get_mod() as i64;
            // let mut u = 1i64;
            // let mut v = 0i64;

            // while b != 0 {
            //     let t = a / b;
            //     a -= t * b;
            //     swap(&mut a, &mut b);
            //     u -= t * v;
            //     swap(&mut u, &mut v);
            // }

            // u %= m;
            // if u < 0 { u += m; }
            // u

            // impl with num_integar::Integar::extended_gcd ...
            let x = self.get().extended_gcd(&(self.get_mod() as i64)).x;
            compensated_rem(x, self.get_mod())
        }
    }

    #[test]
    fn mint_new() {
        let m = ModInt::new(10, 3);
        assert_eq!(m.get(), 1);

        let m = ModInt::new(-10, 3);
        assert_eq!(m.get(), 2);

        let x = 4.to_mint(10); // this is also valid
        let y = ModInt::new(4, 10);
        assert_eq!(x, y);
    }

    #[test]
    fn inv_test() {
        let a = ModInt::new(6, 13);
        assert_eq!(a.inv(), 11);
    }

    impl Add<Self> for ModInt {
        type Output = Self;
        fn add(self, rhs: Self) -> Self::Output {
            let c = check_mod_eq(&self, &rhs);
            if !c.1 {
                panic!("modulo between two instance is different!",)
            }

            let r = self.get() + rhs.num;
            Self {
                num: if r >= self.get_mod() as i64 {
                    r - c.0.get() as i64
                } else {
                    r
                },
                _modulo: Modulo::Static(c.0),
            }
        }
    }

    impl AddAssign<Self> for ModInt {
        fn add_assign(&mut self, rhs: Self) {
            *self = *self + rhs;
        }
    }

    #[test]
    fn mint_add() {
        let a = ModInt::new(13, 8); // 5
        let b = ModInt::new(10, 8); // 2
        assert_eq!((a + b).get(), 7);

        let c = ModInt::new(7, 8); //7
        assert_eq!((a + c).get(), 4); // (5 + 7) % 8 == 4
    }

    impl Sub<Self> for ModInt {
        type Output = Self;
        fn sub(self, rhs: Self) -> Self::Output {
            let c = check_mod_eq(&self, &rhs);
            if !c.1 {
                panic!("modulo between two instance is different!",)
            }
            let num = compensated_rem(self.get() - rhs.get(), c.0.get() as usize);
            Self {
                num,
                _modulo: Modulo::Static(c.0),
            }
        }
    }

    impl SubAssign<Self> for ModInt {
        fn sub_assign(&mut self, rhs: Self) {
            *self = *self - rhs;
        }
    }

    #[test]
    fn mint_sub() {
        let a = ModInt::new(2, 10);
        let b = ModInt::new(3, 10);

        assert_eq!((b - a).get(), 1);
        assert_eq!((a - b).get(), 9);
    }

    impl Mul<Self> for ModInt {
        type Output = Self;
        fn mul(self, rhs: Self) -> Self::Output {
            let c = check_mod_eq(&self, &rhs);
            if !c.1 {
                panic!("modulo between two instance is different!",)
            }
            let num = compensated_rem(self.get() * rhs.get(), c.0.get() as usize);
            Self {
                num,
                _modulo: Modulo::Static(c.0),
            }
        }
    }

    impl MulAssign<Self> for ModInt {
        fn mul_assign(&mut self, rhs: Self) {
            *self = *self * rhs
        }
    }

    impl Div<Self> for ModInt {
        type Output = Self;
        fn div(self, rhs: Self) -> Self::Output {
            let c = check_mod_eq(&self, &rhs);
            if !c.1 {
                panic!("modulo between two instance is different!",)
            }
            Self {
                num: self.get() * rhs.inv() % c.0.get() as i64,
                _modulo: Modulo::Static(c.0),
            }
        }
    }

    impl DivAssign<Self> for ModInt {
        fn div_assign(&mut self, rhs: Self) {
            *self = *self / rhs;
        }
    }

    #[test]
    fn div_test() {
        let a = ModInt::new(2, 5);
        let b = ModInt::new(3, 5);
        assert_eq!(a / b, ModInt::new(4, 5));

        let x = ModInt::new(1, 13);
        assert_eq!((x / 4i64).get(), 10);

        let x = ModInt::new(2, 13);
        assert_eq!((x / 4i64).get(), 7);

        let x = ModInt::new(3, 13);
        assert_eq!((x / 4i64).get(), 4);

        let x = ModInt::new(4, 13);
        assert_eq!((x / 4i64).get(), 1);

        let x = ModInt::new(5, 13);
        assert_eq!((x / 4i64).get(), 11);

        let x = ModInt::new(6, 13);
        assert_eq!((x / 4i64).get(), 8);

        let x = ModInt::new(7, 13);
        assert_eq!((x / 4i64).get(), 5);

        let x = ModInt::new(8, 13);
        assert_eq!((x / 4i64).get(), 2);

        let x = ModInt::new(9, 13);
        assert_eq!((x / 4i64).get(), 12);

        let x = ModInt::new(10, 13);
        assert_eq!((x / 4i64).get(), 9);

        let x = ModInt::new(11, 13);
        assert_eq!((x / 4i64).get(), 6);

        let x = ModInt::new(12, 13);
        assert_eq!((x / 4i64).get(), 3);
    }

    impl Rem for ModInt {
        type Output = Self;
        fn rem(self, rhs: Self) -> Self::Output {
            let c = check_mod_eq(&self, &rhs);
            if !c.1 {
                panic!("modulo between two instance is different!",)
            }
            Self {
                num: self.num % rhs.num,
                _modulo: Modulo::Static(c.0),
            }
        }
    }

    impl RemAssign for ModInt {
        fn rem_assign(&mut self, rhs: Self) {
            *self = *self % rhs
        }
    }

    impl Zero for ModInt {
        fn zero() -> Self {
            ModInt {
                num: 0,
                _modulo: Modulo::Dynamic,
            }
        }
        fn is_zero(&self) -> bool {
            self.num == 0
        }
    }

    impl One for ModInt {
        fn one() -> Self {
            ModInt {
                num: 1,
                _modulo: Modulo::Dynamic,
            }
        }
        fn is_one(&self) -> bool {
            self.num == 1
        }
    }

    impl Num for ModInt {
        type FromStrRadixErr = ParseIntError;
        fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
            let num = str
                .chars()
                .rev()
                .enumerate()
                .map(|(i, b)| radix.pow(i as u32) as i64 * b.to_digit(radix).unwrap() as i64)
                .sum::<i64>();
            Ok(ModInt {
                num,
                _modulo: Modulo::Dynamic,
            })
        }
    }

    impl Pow<usize> for ModInt {
        type Output = Self;
        // fn pow(self, exp: u32) -> Self::Output {
        //     if exp == 0 {
        //         return T::one();
        //     }

        //     while exp & 1 == 0 {
        //         base = base.clone() * base;
        //         exp >>= 1;
        //     }
        //     if exp == 1 {
        //         return base;
        //     }

        //     let mut acc = base.clone();
        //     while exp > 1 {
        //         exp >>= 1;
        //         base = base.clone() * base;
        //         if exp & 1 == 1 {
        //             acc = acc * base.clone();
        //         }
        //     }
        //     acc
        // }

        fn pow(self, exp: usize) -> Self::Output {
            self.pow_mod(exp)
        }
    }

    #[test]
    fn pow_test() {
        let a = ModInt::new(3, 10);
        assert_eq!(a.pow(3).get(), 7);

        let b = ModInt::new(100, 9999);
        assert_eq!(b.pow(2).get(), 1);
    }

    impl Factoriable for ModInt {
        fn falling(self, take: usize) -> Self {
            let mut res = Self::one();
            let mut c = self;
            for _ in 0..take {
                res *= c;
                c -= Self::one();
            }
            res
        }
        fn rising(self, take: usize) -> Self {
            let mut res = Self::one();
            let mut c = self;
            for _ in 0..take {
                res *= c;
                c += 1;
            }
            res
        }
    }

    // #[test]
    // fn fact_test_mint() {
    //     let a = ModInt::new(7, 4);
    //     assert_eq!(a.falling(3).get(), 2);

    //     let a = ModInt::new(6, 7); // 720
    //     assert_eq!(a.factorial().get(), 6);
    // }

    // #[snippet("modint")]
    // impl Integer for ModInt {
    //     fn div_floor(&self, other: &Self) -> Self {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         Self {
    //             num: self.num.div_floor(&other.num),
    //             _modulo: Modulo::Static(c.0),
    //         }
    //     }
    //     fn mod_floor(&self, other: &Self) -> Self {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         Self {
    //             num: self.num.mod_floor(&other.num),
    //             _modulo: Modulo::Static(c.0),
    //         }
    //     }
    //     fn gcd(&self, other: &Self) -> Self {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         Self {
    //             num: self.num.gcd(&other.num),
    //             _modulo: Modulo::Static(c.0),
    //         }
    //     }
    //     fn lcm(&self, other: &Self) -> Self {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         Self {
    //             num: self.num.lcm(&other.num),
    //             _modulo: Modulo::Static(c.0),
    //         }
    //     }
    //     fn divides(&self, other: &Self) -> bool {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         other.num % self.num == 0
    //     }
    //     fn is_multiple_of(&self, other: &Self) -> bool {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         self.num % other.num == 0
    //     }
    //     fn is_even(&self) -> bool {
    //         self.num % 2 == 0
    //     }
    //     fn is_odd(&self) -> bool {
    //         self.num % 2 == 1
    //     }
    //     fn div_rem(&self, other: &Self) -> (Self, Self) {
    //         let c = check_mod_eq(self, other);
    //         if !c.1 {
    //             panic!("modulo between two number is defferent"),;
    //         }
    //         let dr = self.num.div_rem(&other.num);
    //         (
    //             Self {
    //                 num: dr.0,
    //                 _modulo: Modulo::Static(c.0),
    //             },
    //             Self {
    //                 num: dr.1,
    //                 _modulo: Modulo::Static(c.0),
    //             },
    //         )
    //     }
    // }

    macro_rules! impl_ops_between_mint_and_primitive {
    ($($t:ty),*) => {
        $(
            impl Add<$t> for ModInt {
                type Output = Self;
                fn add(self, rhs: $t) -> Self::Output {
                    self + Self::new(rhs as i64, self.get_mod())
                }
            }
            impl AddAssign<$t> for ModInt {
                fn add_assign(&mut self, rhs: $t) {
                    *self = *self + rhs;
                }
            }
            impl Sub<$t> for ModInt {
                type Output = Self;
                fn sub(self, rhs: $t) -> Self::Output {
                    self - Self::new(rhs as i64, self.get_mod())
                }
            }
            impl SubAssign<$t> for ModInt {
                fn sub_assign(&mut self, rhs: $t) {
                    *self = *self - rhs;
                }
            }
            impl Mul<$t> for ModInt {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self::Output {
                    self * Self::new(rhs as i64, self.get_mod())
                }
            }
            impl MulAssign<$t> for ModInt {
                fn mul_assign(&mut self, rhs: $t) {
                    *self = *self * rhs;
                }
            }
            impl Div<$t> for ModInt {
                type Output = Self;
                fn div(self, rhs: $t) -> Self::Output {
                    self / Self::new(rhs as i64, self.get_mod())
                }
            }
            impl DivAssign<$t> for ModInt {
                fn div_assign(&mut self, rhs: $t) {
                    *self = *self / rhs;
                }
            }
        )*
    };
}

    impl_ops_between_mint_and_primitive!(usize, u8, u16, u32, u64, isize, i8, i16, i32, i64);

    #[test]
    fn op_between_different_type() {
        let mut mint = ModInt::new(1, 10);
        mint += 1;
        assert_eq!(mint.get(), 2);
        mint *= 2;
        assert_eq!(mint.get(), 4);
        mint += 10001;
        assert_eq!(mint.get(), 5);
    }
}
