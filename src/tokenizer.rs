use std::fmt;

use regex::Regex;

type SliceVec = Vec<(usize, usize)>;

#[derive(Debug)]
pub enum Pattern {
    Template
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = match *self {
            Pattern::Template => r"(\{%.*?%\}|\{\{.*?\}\}?|\{\{|\{%)"
        };

        write!(f, "{}", result)
    }
}

impl Pattern {
    pub fn to_regex(&self) -> Regex {
        let pattern = format!("{}", self);
        Regex::new(&pattern).unwrap()
    }
}

pub struct Tokenizer<'t> {
    source: &'t str
}

impl<'t> Tokenizer<'t> {
    pub fn new<'a>(source: &'a str) -> Tokenizer<'a> {
        Tokenizer { source: source }
    }

    pub fn tokenize<'a>(&'a self, pattern: &'a Regex) -> Vec<&'a str> {
        let slices = self.matched_slices(pattern);
        slices.iter().map(|&(start, end)| &self.source[start..end]).collect()
    }

    fn matched_slices(&self, pattern: &Regex) -> SliceVec {
        let mut slices = pattern.find_iter(self.source).collect::<Vec<_>>();
        let missing = self.find_missing_slices(&slices);

        slices.extend(&missing);
        slices.sort();
        slices
    }

    fn find_missing_slices(&self, slices: &SliceVec) -> SliceVec {
        if slices.is_empty() { return vec![(0, self.source.len())]; }

        let mut missing = self.missing_middle_slices(slices);

        if let Some(first) = self.missing_first_slice(slices) {
            missing.push(first);
        }

        if let Some(last) = self.missing_last_slice(slices) {
            missing.push(last);
        }

        missing
    }

    fn missing_first_slice(&self, slices: &SliceVec) -> Option<(usize, usize)> {
        let pos = slices[0].0;
        if pos == 0 { return None; }

        Some((0, pos))
    }

    fn missing_last_slice(&self, slices: &SliceVec) -> Option<(usize, usize)> {
        let last = slices.last().unwrap();
        let len  = self.source.len();
        if last.1 == len { return None; }

        Some((last.1, len))
    }

    fn missing_middle_slices(&self, slices: &SliceVec) -> SliceVec {
        let mut missing: SliceVec = Vec::new();
        let last_index            = slices.len() - 1;

        for (index, &(_, right)) in slices.iter().enumerate() {
            if index == last_index { continue; }

            let edge = slices[index + 1].0;
            if right != edge {
                missing.push((right, edge));
            }
        }

        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokens(tokenizer: &Tokenizer, expected: Vec<&str>) {
        let re     = Pattern::Template.to_regex();
        let actual = tokenizer.tokenize(&re);

        assert_eq!(expected, actual);
    }

    #[test]
    fn tokenize_blank_string() {
        let tokenizer = Tokenizer::new("");
        assert_tokens(&tokenizer, vec![""]);
    }

    #[test]
    fn tokenize_whitespace_only_string() {
        let tokenizer = Tokenizer::new("  ");
        assert_tokens(&tokenizer, vec!["  "]);
    }

    #[test]
    fn tokenize_string_with_no_matches() {
        let tokenizer = Tokenizer::new("hello world");
        assert_tokens(&tokenizer, vec!["hello world"]);
    }

    #[test]
    fn tokenize_single_variable() {
        let tokenizer = Tokenizer::new("{{funk}}");
        assert_tokens(&tokenizer, vec!["{{funk}}"]);
    }

    #[test]
    fn tokenize_single_variable_surrounded_by_whitespace() {
        let tokenizer = Tokenizer::new(" {{funk}} ");
        assert_tokens(&tokenizer, vec![" ", "{{funk}}", " "]);
    }

    #[test]
    fn tokenize_multiple_variables() {
        let tokenizer = Tokenizer::new(" {{funk}} {{so}} {{brutha}} ");
        let expected = vec![
            " ",
            "{{funk}}",
            " ",
            "{{so}}",
            " ",
            "{{brutha}}",
            " "
        ];

        assert_tokens(&tokenizer, expected);
    }

    #[test]
    fn tokenize_single_block() {
        let tokenizer = Tokenizer::new(" {%comment%} ");
        assert_tokens(&tokenizer, vec![" ", "{%comment%}", " "]);
    }

    #[test]
    fn tokenize_empty_block_tag() {
        let tokenizer = Tokenizer::new(" {% thing %} {% comment %} My comment here {% endcomment %} ");

        assert_tokens(&tokenizer, vec![
            " ",
            "{% thing %}",
            " ",
            "{% comment %}",
            " My comment here ",
            "{% endcomment %}",
            " "
        ]);
    }

    #[test]
    fn tokenize_multiline_string() {
        let tokenizer = Tokenizer::new("{%comment%}\nMy Comment\n{%endcomment%}\n");

        assert_tokens(&tokenizer, vec![
            "{%comment%}",
            "\nMy Comment\n",
            "{%endcomment%}",
            "\n"
        ]);
    }

    #[test]
    fn tokenize_html_with_liquid() {
        let content = r#"
<html>
  <head>
    <title>{{ title }}</title>
  </head>
  <body class="some-class">
    <p>{% comment %}Content here{% endcomment %}</p>
    <script type="text/javascript">
      var {{ name }} = function() {
        alert("{{ js_value }}");
      };
    </script>
  </body>
</html>
        "#;

        let tokenizer = Tokenizer::new(&content);
        assert_tokens(&tokenizer, vec![
            "\n<html>\n  <head>\n    <title>",
            "{{ title }}",
            "</title>\n  </head>\n  <body class=\"some-class\">\n    <p>",
            "{% comment %}",
            "Content here",
            "{% endcomment %}",
            "</p>\n    <script type=\"text/javascript\">\n      var ",
            "{{ name }}",
            " = function() {\n        alert(\"",
            "{{ js_value }}",
            "\");\n      };\n    </script>\n  </body>\n</html>\n        "
        ]);
    }
}
