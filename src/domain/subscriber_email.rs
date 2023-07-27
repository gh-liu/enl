#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validator::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "liushgoogle.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@google.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::quickcheck;

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail().fake();
        claims::assert_ok!(SubscriberEmail::parse(email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    quickcheck! {
        fn valid_emails_are_always_parsed_successfully(email: ValidEmailFixture) -> bool {
            SubscriberEmail::parse(email.0).is_ok()
        }
    }
}
