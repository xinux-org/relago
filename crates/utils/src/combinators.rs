trait Alternative: Sized {
    fn empty() -> Self;
    fn alt(self, other: Self) -> Self;
}

impl<T> Alternative for Option<T> {
    fn empty() -> Self {
        None
    }

    fn alt(self, other: Self) -> Self {
        match self {
            Some(x) => Some(x),
            None => other,
        }
    }
}

impl<T> Alternative for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }

    fn alt(mut self, mut other: Self) -> Self {
        self.append(&mut other);
        self
    }
}

impl<T, E> Alternative for Result<T, E> {
    fn empty() -> Self {
        panic!("Result::empty() not supported")
    }

    fn alt(self, other: Self) -> Self {
        match self {
            Ok(x) => Ok(x),
            Err(_) => other,
        }
    }
}
// Macro to provide <|> syntax
macro_rules! alt {
    ($a:expr) => { $a };
    ($a:expr, $($rest:expr),+) => {
        $a.alt(alt!($($rest),+))
    };
}

#[cfg(test)]
mod macro_tests {
    use super::*;

    #[test]
    fn single_argument() {
        let result = alt!(Some(42));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn single_argument_none() {
        let result: Option<i32> = alt!(None);
        assert_eq!(result, None);
    }

    #[test]
    fn two_arguments() {
        let result = alt!(None::<i32>, Some(42));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn two_arguments_both_some() {
        let result = alt!(Some(1), Some(2));
        assert_eq!(result, Some(1));
    }

    #[test]
    fn three_arguments() {
        let result = alt!(None::<i32>, None, Some(42));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn four_arguments() {
        let result = alt!(None::<i32>, None, Some(42), Some(100));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn five_arguments() {
        let result = alt!(None::<i32>, None, None, None, Some(99));
        assert_eq!(result, Some(99));
    }

    #[test]
    fn many_arguments() {
        let result = alt!(
            None::<i32>,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(777),
            Some(888)
        );
        assert_eq!(result, Some(777));
    }

    // ----- Macro with Different Types -----

    #[test]
    fn strings() {
        let result = alt!(None::<String>, None, Some("hello".to_string()));
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn str_slices() {
        let result = alt!(None::<&str>, Some("world"), Some("hello"));
        assert_eq!(result, Some("world"));
    }

    #[test]
    fn vectors() {
        let result = alt!(vec![1, 2], vec![3, 4], vec![5, 6]);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn empty_vectors() {
        let result = alt!(Vec::<i32>::new(), vec![1], vec![2, 3]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn results() {
        let result: Result<i32, &str> = alt!(Err("e1"), Err("e2"), Ok(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn results_multiple_ok() {
        let result: Result<i32, &str> = alt!(Err("e1"), Ok(10), Ok(20));
        assert_eq!(result, Ok(10));
    }

    // ----- Macro with Complex Expressions -----

    #[test]
    fn function_calls() {
        fn get_none() -> Option<i32> {
            None
        }
        fn get_some() -> Option<i32> {
            Some(42)
        }

        let result = alt!(get_none(), get_none(), get_some());
        assert_eq!(result, Some(42));
    }

    #[test]
    fn inline_expressions() {
        let x = 10;
        let result = alt!(None, Some(x * 2), Some(x * 3));
        assert_eq!(result, Some(20));
    }

    #[test]
    fn closures() {
        let get_value = || Some(42);
        let result = alt!(None::<i32>, get_value());
        assert_eq!(result, Some(42));
    }

    #[test]
    fn method_calls() {
        let s = "42";
        let result = alt!("not a number".parse::<i32>().ok(), s.parse::<i32>().ok());
        assert_eq!(result, Some(42));
    }

    #[test]
    fn conditional_expressions() {
        let flag = true;
        let result = alt!(None::<i32>, if flag { Some(100) } else { None });
        assert_eq!(result, Some(100));
    }

    // ----- Macro with Nested Calls -----

    #[test]
    fn nested_simple() {
        let result = alt!(None::<i32>, alt!(None, Some(42)));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn nested_complex() {
        let result = alt!(None::<i32>, alt!(None, None), alt!(Some(42), Some(100)));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn deeply_nested() {
        let result = alt!(None::<i32>, alt!(None, alt!(None, Some(999))));
        assert_eq!(result, Some(999));
    }

    // ----- Macro Edge Cases -----

    #[test]
    fn all_none() {
        let result: Option<i32> = alt!(None, None, None, None);
        assert_eq!(result, None);
    }

    #[test]
    fn all_some() {
        let result = alt!(Some(1), Some(2), Some(3));
        assert_eq!(result, Some(1));
    }

    #[test]
    fn empty_vectors_only() {
        let result = alt!(Vec::<i32>::new(), Vec::<i32>::new(), Vec::<i32>::new());
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn single_element_vectors() {
        let result = alt!(vec![1], vec![2], vec![3]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    // ----- Macro with Type Inference -----

    #[test]
    fn infers_type_from_context() {
        let result: Option<i32> = alt!(None, None, Some(42));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn infers_complex_type() {
        #[derive(Debug, PartialEq)]
        struct Data {
            value: i32,
        }

        let result = alt!(None, Some(Data { value: 42 }));
        assert_eq!(result, Some(Data { value: 42 }));
    }

    // ----- Macro Evaluation Order -----

    #[test]
    fn left_to_right_evaluation() {
        use std::cell::RefCell;

        let order = RefCell::new(Vec::new());

        let f1 = || {
            order.borrow_mut().push(1);
            None::<i32>
        };
        let f2 = || {
            order.borrow_mut().push(2);
            None::<i32>
        };
        let f3 = || {
            order.borrow_mut().push(3);
            Some(42)
        };

        let _result = alt!(f1(), f2(), f3());

        assert_eq!(*order.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn stops_evaluation_on_first_success() {
        use std::cell::RefCell;

        let call_count = RefCell::new(0);

        let success = || {
            *call_count.borrow_mut() += 1;
            Some(42)
        };
        let after = || {
            *call_count.borrow_mut() += 1;
            Some(100)
        };

        let _result = alt!(success(), after());

        // Both should be called because macro evaluates all arguments
        assert_eq!(*call_count.borrow(), 2);
    }

    // ----- Macro with Variables -----

    #[test]
    fn mixed_literals_and_variables() {
        let x = Some(100);
        let result = alt!(None::<i32>, x, Some(200));
        assert_eq!(result, Some(100));
    }

    // ----- Macro with Different Integer Types -----

    #[test]
    fn u32() {
        let result = alt!(None::<u32>, Some(42u32));
        assert_eq!(result, Some(42u32));
    }

    #[test]
    fn i64() {
        let result = alt!(None::<i64>, Some(9999999999i64));
        assert_eq!(result, Some(9999999999i64));
    }

    #[test]
    fn usize() {
        let result = alt!(None::<usize>, None, Some(42usize));
        assert_eq!(result, Some(42usize));
    }

    // ----- Macro Practical Patterns -----

    #[test]
    fn parser_pattern() {
        fn parse_decimal(s: &str) -> Option<i32> {
            s.parse().ok()
        }

        fn parse_hex(s: &str) -> Option<i32> {
            if s.starts_with("0x") {
                i32::from_str_radix(&s[2..], 16).ok()
            } else {
                None
            }
        }

        fn parse_binary(s: &str) -> Option<i32> {
            if s.starts_with("0b") {
                i32::from_str_radix(&s[2..], 2).ok()
            } else {
                None
            }
        }

        let input = "0xFF";
        let result = alt!(parse_decimal(input), parse_hex(input), parse_binary(input));
        assert_eq!(result, Some(255));
    }

    #[test]
    fn fallback_chain() {
        fn primary() -> Option<String> {
            None
        }
        fn secondary() -> Option<String> {
            None
        }
        fn tertiary() -> Option<String> {
            Some("fallback".to_string())
        }

        let result = alt!(primary(), secondary(), tertiary());
        assert_eq!(result, Some("fallback".to_string()));
    }

    // ----- Macro with Collections -----

    #[test]
    fn string_vectors() {
        let result = alt!(
            vec!["a".to_string()],
            vec!["b".to_string()],
            vec!["c".to_string()]
        );
        assert_eq!(
            result,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn mixed_vector_sizes() {
        let result = alt!(vec![1], vec![2, 3], vec![4, 5, 6]);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn long_vector_chain() {
        let result = alt!(
            vec![1],
            vec![2],
            vec![3],
            vec![4],
            vec![5],
            vec![6],
            vec![7],
            vec![8],
            vec![9],
            vec![10]
        );
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
}
