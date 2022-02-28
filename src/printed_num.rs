#[derive(Eq, PartialEq)]
pub struct PrintedNum<'a>(pub &'a str);

impl std::fmt::Display for PrintedNum<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl num_traits::Num for PrintedNum<'_> {
    type FromStrRadixErr = std::convert::Infallible;

    fn from_str_radix(_: &str, _: u32) -> Result<Self, Self::FromStrRadixErr> {
        panic!("not defined")
    }
}

impl num_traits::Zero for PrintedNum<'_> {
    fn zero() -> Self {
        PrintedNum("0")
    }

    fn is_zero(&self) -> bool {
        self.0 == "0"
    }
}

impl num_traits::One for PrintedNum<'_> {
    fn one() -> Self {
        PrintedNum("1")
    }
}

macro_rules! impl_operator(($trait:path, $method:ident) => {
    impl $trait for PrintedNum<'_> {
        type Output = Self;

        fn $method(self, _: Self) -> Self::Output {
            panic!("not defined")
        }
    }
});

impl_operator!(std::ops::Add, add);
impl_operator!(std::ops::Sub, sub);
impl_operator!(std::ops::Mul, mul);
impl_operator!(std::ops::Div, div);
impl_operator!(std::ops::Rem, rem);
