#[cfg(test)]
pub mod utils {
    use regex::Regex;

    use proc_macro2::TokenStream;
    use std::str::FromStr;

    fn normalize(code: &str) -> String {
        TokenStream::from_str(code)
            .expect("Failed to parse code as TokenStream")
            .to_string()
    }

    fn tokenize(code: &str) -> Vec<String> {
        let ts = TokenStream::from_str(code).expect("Failed to tokenize string for debugging");

        ts.into_iter().map(|token| token.to_string()).collect()
    }

    fn build_matcher(expected: &str) -> Regex {
        let normalized = normalize(expected);
        let mut pattern = regex::escape(&normalized);
        pattern = pattern.replace(r"\ ", r"\s*");

        pattern = pattern.replace(r"\.\*", r".*");

        let full_pattern = format!(r"^{}$", pattern);
        Regex::new(&full_pattern).expect("Failed to build regex pattern")
    }

    fn compare_tokens(actual: &str, expected: &str) {
        let actual_norm = normalize(&actual);
        let matcher = build_matcher(expected);

        if !matcher.is_match(&actual_norm) {
            let mut partial_match = String::new();
            let tokens = tokenize(expected);

            for i in 1..=tokens.len() {
                let sub_expected = tokens[..i].join(" ");
                let sub_matcher = build_matcher(&sub_expected);
                let prefix_re = Regex::new(&format!(
                    r"^\s*{}",
                    sub_matcher
                        .as_str()
                        .trim_start_matches('^')
                        .trim_end_matches('$')
                ))
                .unwrap();

                if prefix_re.is_match(&actual_norm) {
                    partial_match = sub_expected;
                } else {
                    break;
                }
            }

            panic!(
                "\n\x1b[31mRegex Mismatch!\x1b[0m\n\n\
             \x1b[1mStopped matching after:\x1b[0m {:?}\n\
             \x1b[1mNext expected token was:\x1b[0m {:?}\n\n\
             \x1b[1mActual (normalized):\x1b[0m   {:?}\n\
             \x1b[1mFull Expected Pattern:\x1b[0m {:?}\n",
                partial_match,
                tokens
                    .get(partial_match.split_whitespace().count())
                    .unwrap_or(&"END".to_string()),
                actual_norm,
                expected
            );
        }
    }

    pub fn verify_expansion(actual_raw: String, expected: &str) {
        compare_tokens(&actual_raw, expected);
    }

    #[macro_export]
    macro_rules! assert_expansion_regex {
        ($impl:ident, $input:item => $exp:expr) => {{
            let out = $impl(parse_quote!($input)).to_string();
            verify_expansion(out, $exp);
        }};
    }

    #[macro_export]
    macro_rules! assert_expansion {
        ($impl_func:ident, $input_fn:item => $expected_contains:expr) => {
            let input: ItemFn = parse_quote! { $input_fn };
            let output = $impl_func(input).to_string();

            let normalized_output = output.replace(" ", "");
            let expected_str = $expected_contains.replace(" ", "");

            assert!(
                normalized_output.contains(&expected_str),
                "\nExpansion mismatch!\nOutput: {}\nExpected to contain: {}\n",
                output,
                $expected_contains
            );
        };
    }
}
