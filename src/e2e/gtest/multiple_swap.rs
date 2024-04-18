use crate::test_helpers::gtest::multiple_swap;

#[test]
pub fn test_multiple_swap_x_to_y() {
    multiple_swap(true);
}

#[test]
pub fn test_multiple_swap_y_to_x() {
    multiple_swap(false);
}
