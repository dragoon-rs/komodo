use rand::{Rng, RngCore};

use crate::random::draw_unique_elements;

#[derive(Debug, PartialEq)]
pub(super) enum Strategy {
    Single { n: usize },
    Double { p: f64, n: usize, m: usize },
}

impl Strategy {
    pub(super) fn draw<T: Clone>(&self, things: &[T], rng: &mut impl RngCore) -> Vec<T> {
        let nb_to_take = match self {
            Self::Single { n } => *n,
            Self::Double { p, n, m } => {
                if rng.gen::<f64>() < *p {
                    *n
                } else {
                    *m
                }
            }
        };

        draw_unique_elements(things, nb_to_take, rng)
    }

    pub(super) fn from_str(s: &str) -> Result<Self, String> {
        if !s.contains(':') {
            return Err(format!(
                "expected at least one :-separated token in '{}', found 0",
                s,
            ));
        }

        let tokens: Vec<&str> = s.split(':').collect();

        match tokens[0] {
            "single" => {
                if tokens.len() != 2 {
                    return Err(format!(
                        "expected 2 :-separated tokens in '{}', found {}",
                        s,
                        tokens.len(),
                    ));
                }

                let n = match tokens[1].parse::<usize>() {
                    Ok(u) => u,
                    Err(_) => {
                        return Err(format!(
                            "could not parse positive integer from '{}'",
                            tokens[1]
                        ))
                    }
                };
                Ok(Self::Single { n })
            }
            "double" => {
                if tokens.len() != 4 {
                    return Err(format!(
                        "expected 4 :-separated tokens in '{}', found {}",
                        s,
                        tokens.len(),
                    ));
                }

                let p = match tokens[1].parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => return Err(format!("could not parse float from '{}'", tokens[1])),
                };
                if !(0.0..=1.0).contains(&p) {
                    return Err(format!("p should be a probability, found {}", p));
                }

                let n = match tokens[2].parse::<usize>() {
                    Ok(u) => u,
                    Err(_) => {
                        return Err(format!(
                            "could not parse positive integer from '{}'",
                            tokens[2]
                        ))
                    }
                };
                let m = match tokens[3].parse::<usize>() {
                    Ok(u) => u,
                    Err(_) => {
                        return Err(format!(
                            "could not parse positive integer from '{}'",
                            tokens[3]
                        ))
                    }
                };

                Ok(Self::Double { p, n, m })
            }
            ty => Err(format!("unknown strat type '{}'", ty)),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn strategy() {
        assert_eq!(
            super::Strategy::from_str("single:3"),
            Ok(super::Strategy::Single { n: 3 })
        );
        assert_eq!(
            super::Strategy::from_str("double:0.1:1:2"),
            Ok(super::Strategy::Double { p: 0.1, n: 1, m: 2 })
        );

        let cases = vec![
            (
                "foo",
                "expected at least one :-separated token in 'foo', found 0",
            ),
            ("foo:bar:baz", "unknown strat type 'foo'"),
            ("single:", "could not parse positive integer from ''"),
            (
                "single::",
                "expected 2 :-separated tokens in 'single::', found 3",
            ),
            (
                "double:bar:baz",
                "expected 4 :-separated tokens in 'double:bar:baz', found 3",
            ),
            (
                "double:bar:baz:spam:eggs",
                "expected 4 :-separated tokens in 'double:bar:baz:spam:eggs', found 5",
            ),
            ("double:bar::", "could not parse float from 'bar'"),
            ("double:1.2::", "p should be a probability, found 1.2"),
            ("double:0.2::", "could not parse positive integer from ''"),
            ("double:0.2:1:", "could not parse positive integer from ''"),
        ];

        for (input, expected_error) in cases {
            assert_eq!(
                super::Strategy::from_str(input),
                Err(expected_error.to_string()),
                "input: {}",
                input,
            );
        }
    }
}
