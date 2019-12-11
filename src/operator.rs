use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub(super) enum Operator {
    ParenLeft = 1 << 1,
    ParenRight = (1 << 1) + 1,

    Not = 1 << 2,
    Neg = (1 << 2) + 1,

    Mul = 1 << 3,
    Div = (1 << 3) + 1,
    Rem = (1 << 3) + 2,

    Add = 1 << 4,
    Sub = (1 << 4) + 1,

    Eq = 1 << 5,
    Ne = (1 << 5) + 1,
    Gt = (1 << 5) + 2,
    Lt = (1 << 5) + 3,
    Ge = (1 << 5) + 4,
    Le = (1 << 5) + 5,

    And = 1 << 6,
    Or = (1 << 6) + 1,
}

impl Operator {
    fn preference(self, o: Operator) -> Ordering {
        (self as u8).leading_zeros().cmp(&(o as u8).leading_zeros())
    }

    pub(super) fn gt_preference(self, o: Operator) -> bool {
        match self.preference(o) {
            Ordering::Greater => true,
            _ => false,
        }
    }

    pub(super) fn eq_preference(self, o: Operator) -> bool {
        match self.preference(o) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_op_preference() {
        assert!(!Operator::Add.gt_preference(Operator::Sub) && Operator::Sub != Operator::Add);
        assert!(!Operator::Not.gt_preference(Operator::Not));
        assert!(!Operator::Add.gt_preference(Operator::Not));
        assert!(Operator::Not.gt_preference(Operator::Add));
    }
}
