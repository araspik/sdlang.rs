use super::gen;
use super::utils::RuleSet;
use crate::grammar::Rule;
use crate::parse;

use proptest::prelude::*;

proptest! {
    #[test]
    fn string(test in gen::string()) {
        RuleSet::new(Rule::string, parse::string).test(test)
    }

    #[test]
    fn date(test in gen::date()) {
        RuleSet::new(Rule::date, parse::date).test(test)
    }

    #[test]
    fn time(test in gen::time()) {
        RuleSet::new(Rule::time, parse::time).test(test)
    }

    #[test]
    fn datetime(test in gen::datetime()) {
        RuleSet::new(Rule::datetime, parse::datetime).test(test)
    }

    #[test]
    fn duration(test in gen::duration()) {
        RuleSet::new(Rule::duration, parse::duration).test(test)
    }

    #[test]
    fn number(test in gen::number()) {
        RuleSet::new(Rule::number, parse::number).test(test)
    }

    #[test]
    fn decimal(test in gen::decimal()) {
        RuleSet::new(Rule::decimal, parse::decimal).test(test)
    }

    #[test]
    fn boolean(test in gen::boolean()) {
        RuleSet::new(Rule::boolean, parse::boolean).test(test)
    }

    #[test]
    fn base64(test in gen::base64()) {
        RuleSet::new(Rule::base64, parse::base64).test(test)
    }

    #[test]
    fn value(test in gen::value()) {
        RuleSet::new(Rule::value, parse::value).test(test)
    }

    #[test]
    fn ident(test in gen::ident()) {
        RuleSet::new(Rule::ident, parse::ident).test(test)
    }

    #[test]
    fn namespace(test in gen::namespace()) {
        RuleSet::new(Rule::namespace, parse::namespace).test(test)
    }

    #[test]
    fn attribute(test in gen::attribute()) {
        RuleSet::new(Rule::attribute, parse::attribute).test(test)
    }

    #[test]
    fn tag(test in gen::tag()) {
        RuleSet::new(Rule::tag, parse::tag).test(test)
    }

    #[test]
    fn tagtree(test in gen::tagtree()) {
        RuleSet::new(Rule::tagtree, parse::tagtree).test(test)
    }
}
