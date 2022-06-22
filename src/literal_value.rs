use serde::{de::Visitor, Deserialize};

macro_rules! literal_bool {
    ($src:literal, $dst:ident) => {
        #[derive(PartialEq, Eq)]
        pub struct $dst;

        impl std::fmt::Debug for $dst {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", $src)
            }
        }

        impl<'de> Deserialize<'de> for $dst {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct LiteralVisitor;

                impl<'de> Visitor<'de> for LiteralVisitor {
                    type Value = $dst;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(&format!("bool `{}`", $src))
                    }

                    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v == $src {
                            Ok($dst)
                        } else {
                            Err(E::custom(format!("must be bool `{}`", $src)))
                        }
                    }
                }

                deserializer.deserialize_bool(LiteralVisitor)
            }
        }
    };
}

literal_bool!(true, LiteralTrue);
literal_bool!(false, LiteralFalse);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_literal_true() {
        assert_eq!(format!("{:?}", LiteralTrue), "true");
        assert_eq!(
            serde_yaml::from_str::<LiteralTrue>("true").unwrap(),
            LiteralTrue
        );
        assert!(serde_yaml::from_str::<LiteralTrue>("false").is_err())
    }

    #[test]
    fn test_literal_false() {
        assert_eq!(format!("{:?}", LiteralFalse), "false");
        assert_eq!(
            serde_yaml::from_str::<LiteralFalse>("false").unwrap(),
            LiteralFalse
        );
        assert!(serde_yaml::from_str::<LiteralFalse>("true").is_err())
    }
}
