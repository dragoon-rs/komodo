use rand::{Rng, RngCore};

use crate::random::draw_unique_elements;

#[derive(Debug, PartialEq)]
pub(super) enum Environment {
    /// at each time step, remove `n` elements with probability `p`
    RandomFixed { p: f64, n: usize },
    /// at each time step, remove a fraction `q` of the elements with probability `p`
    RandomDynamic { p: f64, q: f64 },
    /// at each time step, remove `n` elements
    Fixed { n: usize },
}

impl Environment {
    /// `update(things, rng)` is `things` with some elements potentially removed according to the
    /// [`Environment`] type
    pub(super) fn update<T: Clone>(&self, things: &[T], rng: &mut impl RngCore) -> Vec<T> {
        let nb_to_take = match self {
            Environment::Fixed { n } => things.len() - n,
            Environment::RandomFixed { p, n } => {
                if rng.gen::<f64>() > *p {
                    return things.to_vec();
                }
                things.len() - n
            }
            Environment::RandomDynamic { p, q } => {
                if rng.gen::<f64>() > *p {
                    return things.to_vec();
                }
                (things.len() as f64 * (1.0 - q)) as usize
            }
        };

        draw_unique_elements(things, nb_to_take, rng)
    }

    pub(super) fn from_str(s: &str) -> Result<Self, String> {
        let tokens: Vec<&str> = s.split(':').collect();
        if tokens.is_empty() {
            return Err(format!(
                "expected at least one :-separated token in '{}', found 0",
                s,
            ));
        }

        match tokens[0] {
            "fixed" => {
                let n = match tokens[1].parse::<usize>() {
                    Ok(u) => u,
                    Err(_) => {
                        return Err(format!(
                            "could not parse positive integer from '{}'",
                            tokens[1]
                        ))
                    }
                };
                Ok(Environment::Fixed { n })
            }
            "random-fixed" => {
                if tokens.len() != 3 {
                    return Err(format!(
                        "expected 3 :-separated tokens in '{}', found {}",
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
                Ok(Environment::RandomFixed { p, n })
            }
            "random-dynamic" => {
                if tokens.len() != 3 {
                    return Err(format!(
                        "expected 3 :-separated tokens in '{}', found {}",
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

                let q = match tokens[2].parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => return Err(format!("could not parse float from '{}'", tokens[2])),
                };
                if !(0.0..=1.0).contains(&q) {
                    return Err(format!("q should be between 0 and 1, found {}", q));
                }

                Ok(Environment::RandomDynamic { p, q })
            }
            ty => Err(format!("unknow env type '{}'", ty)),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn environment() {
        assert_eq!(
            super::Environment::from_str("fixed:1"),
            Ok(super::Environment::Fixed { n: 1 })
        );
        assert_eq!(
            super::Environment::from_str("random-fixed:0.2:1"),
            Ok(super::Environment::RandomFixed { p: 0.2, n: 1 })
        );
        assert_eq!(
            super::Environment::from_str("random-dynamic:0.2:0.3"),
            Ok(super::Environment::RandomDynamic { p: 0.2, q: 0.3 })
        );

        let cases = vec![
            ("foo", "unknow env type 'foo'"),
            ("foo:", "unknow env type 'foo'"),
            ("fixed:", "could not parse positive integer from ''"),
            ("fixed:foo", "could not parse positive integer from 'foo'"),
            (
                "random-fixed:",
                "expected 3 :-separated tokens in 'random-fixed:', found 2",
            ),
            ("random-fixed:foo:", "could not parse float from 'foo'"),
            ("random-fixed:1.2:", "p should be a probability, found 1.2"),
            (
                "random-fixed:0.2:",
                "could not parse positive integer from ''",
            ),
            (
                "random-fixed:0.2:foo",
                "could not parse positive integer from 'foo'",
            ),
            (
                "random-dynamic:",
                "expected 3 :-separated tokens in 'random-dynamic:', found 2",
            ),
            ("random-dynamic:foo:", "could not parse float from 'foo'"),
            (
                "random-dynamic:1.2:",
                "p should be a probability, found 1.2",
            ),
            ("random-dynamic:0.2:", "could not parse float from ''"),
            ("random-dynamic:0.2:foo", "could not parse float from 'foo'"),
            (
                "random-dynamic:0.2:1.2",
                "q should be between 0 and 1, found 1.2",
            ),
        ];

        for (input, expected_error) in cases {
            assert_eq!(
                super::Environment::from_str(input),
                Err(expected_error.to_string()),
                "input: {}",
                input
            );
        }
    }
}
