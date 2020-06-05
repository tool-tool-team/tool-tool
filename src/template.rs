use crate::{bail, Result};
use anyhow::Context;

pub fn template<F: Fn(&str) -> Result<String>>(string: &str, replacer: F) -> Result<String> {
    //    replacer(string)
    let mut result = String::new();
    let mut haystack = string;
    while let Some(start) = haystack.find("${") {
        result += &haystack[..start];
        haystack = &haystack[start + 2..];
        match haystack.find("}") {
            None => bail!(
                "Unclosed template string in template '{}', you may be missing a closing '}}'",
                string
            ),
            Some(end) => {
                let name = &haystack[..end];
                result += &replacer(name).with_context(|| {
                    format!(
                        "Could not replace template name '{}' in template string {}",
                        name, string
                    )
                })?;
                haystack = &haystack[end + 1..];
            }
        }
    }
    result += haystack;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replacer(string: &str) -> Result<String> {
        Ok(string.to_uppercase())
    }

    fn template_test(input: &str, output: &str) {
        assert_eq!(template(input, replacer).unwrap(), output);
    }
    #[test]
    fn template_nothing() {
        template_test("foobar", "foobar");
    }

    #[test]
    fn template_everything() {
        template_test("${foobar}", "FOOBAR");
    }

    #[test]
    fn template_mixed() {
        template_test("a${x}b${y}c", "aXbYc");
    }

    #[test]
    fn template_mixed2() {
        template_test("${x}b${y}", "XbY");
    }

    #[test]
    fn template_missing() {
        let error = template("${foo}", |_| bail!("Failhard")).expect_err("Want error");
        assert_eq!(
            error.to_string(),
            "Could not replace template name 'foo' in template string ${foo}"
        );
        assert_eq!(error.source().expect("cause").to_string(), "Failhard");
    }

    #[test]
    fn template_unclosed() {
        assert_eq!(
            template("${foo", |_| Ok("never".into()))
                .expect_err("Want error")
                .to_string(),
            "Unclosed template string in template '${foo', you may be missing a closing '}'"
        );
    }
}
