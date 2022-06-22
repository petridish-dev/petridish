use serde::Deserialize;

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
                use serde::de::Visitor;

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

macro_rules! literal_str {
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
                use serde::de::Visitor;
                struct LiteralVisitor;

                impl<'de> Visitor<'de> for LiteralVisitor {
                    type Value = $dst;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(&format!("str `{}`", $src))
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v == $src {
                            Ok($dst)
                        } else {
                            Err(E::custom(format!("must be str `{}`", $src)))
                        }
                    }
                }

                deserializer.deserialize_str(LiteralVisitor)
            }
        }
    };
}

pub(crate) use literal_str;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_literal_true() {
        assert_eq!(format!("{:?}", LiteralTrue), "true");
        assert_eq!(
            serde_json::from_str::<LiteralTrue>("true").unwrap(),
            LiteralTrue
        );
        assert!(serde_json::from_str::<LiteralTrue>("false").is_err())
    }

    #[test]
    fn test_literal_false() {
        assert_eq!(format!("{:?}", LiteralFalse), "false");
        assert_eq!(
            serde_json::from_str::<LiteralFalse>("false").unwrap(),
            LiteralFalse
        );
        assert!(serde_json::from_str::<LiteralFalse>("true").is_err())
    }

    #[test]
    fn test_literal_str() {
        literal_str!("str", LiteralStr);
        assert_eq!(format!("{:?}", LiteralStr), "str");
        assert_eq!(
            serde_json::from_str::<LiteralStr>("\"str\"").unwrap(),
            LiteralStr
        );
        assert!(serde_json::from_str::<LiteralStr>("\"string\"").is_err())
    }
}
