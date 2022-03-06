use crate::language::Language;
use anyhow::{Context, Result};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Point, Query, QueryCursor, QueryPredicateArg};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Extractor {
    language: Language,
    ts_language: tree_sitter::Language,
    query: Query,
    captures: Vec<String>,
    ignores: HashSet<usize>,
}

impl Extractor {
    pub fn new(language: Language, query: Query) -> Extractor {
        let captures = query.capture_names().to_vec();

        let mut ignores = HashSet::default();
        captures.iter().enumerate().for_each(|(i, name)| {
            if name.starts_with('_') {
                ignores.insert(i);
            }
        });

        Extractor {
            ts_language: (&language).language(),
            language,
            query,
            captures,
            ignores,
        }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn extract_from_file(
        &self,
        path: &Path,
        parser: &mut Parser,
    ) -> Result<Option<ExtractedFile>> {
        let source = fs::read(&path).context("could not read file")?;

        self.extract_from_text(Some(path), &source, parser)
    }

    pub fn extract_from_text(
        &self,
        path: Option<&Path>,
        source: &[u8],
        parser: &mut Parser,
    ) -> Result<Option<ExtractedFile>> {
        parser
            .set_language(self.ts_language)
            .context("could not set language")?;

        let tree = parser
            .parse(&source, None)
            // note: this could be a timeout or cancellation, but we don't set
            // that so we know it's always a language error. Buuuut we also
            // always set the language above so if this happens we also know
            // it's an internal error.
            .context(
                "could not parse to a tree. This is an internal error and should be reported.",
            )?;
	
	// println!("{:?}", &self.query);
        let mut to_substitute = HashMap::new();
	for p in self.query.general_predicates(0) {
		match &*p.operator {
			"sub!" => {
				//println!("{:?}", &p);
				let mut key: usize = 0;
				let mut value: String = "".to_string();
				for a in &p.args { 
					match a {
						QueryPredicateArg::Capture(c) => {
							key = (1 + *c) as usize;
						},
						QueryPredicateArg::String(s) => {
							value = s.to_string();
						}
					}
				}
				if key != 0 {
					to_substitute.insert(key, value);
				}
			},
			_ => {}
		}
	}
        //println!("{:?}", to_substitute);
        let mut name_to_key = HashMap::new();
	let mut cursor = QueryCursor::new();
        let extracted_matches = cursor
            .matches(&self.query, tree.root_node(), source)
            .flat_map(|query_match| query_match.captures)
            // note: the casts here could potentially break if run on a 16-bit
            // microcontroller. I don't think this is a huge problem, though,
            // since even the gnarliest queries I've written have something on
            // the order of 20 matches. Nowhere close to 2^16!
            .map(|capture| {
                let name = &self.captures[capture.index as usize];
		name_to_key.insert(name.to_string(), (capture.index +1) as usize);
                let node = capture.node;
                let text = match node
                    .utf8_text(source)
                    .map(|unowned| unowned.to_string())
                    .context("could not extract text from capture")
                {
                    Ok(text) => { text },
                    Err(problem) => return Err(problem),
                };
                Ok(ExtractedMatch {
                    kind: node.kind(),
                    name,
                    text,
                    start: node.start_position(),
                    end: node.end_position(),
                })
            })
            .collect::<Result<Vec<ExtractedMatch>>>()?;

        if extracted_matches.is_empty() {
            Ok(None)
        } else {
	    let mut extracted_matches2 = Vec::<ExtractedMatch>::new();
	    for m in &extracted_matches {
		let k = name_to_key.get(&m.name.to_string()).unwrap();
		let mut text = String::from(&m.text);
		if to_substitute.contains_key(k) {
			text = to_substitute.get(k).unwrap().to_string();
			for m1 in &extracted_matches {
			    let from_str = String::from("@") + m1.name;
			    let from = from_str.as_str();
			    let to = m1.text.as_str();
			    text = text.replace(from, to);
			}
		}
		if ! self.ignores.contains(&(k-1 as usize)) {
			extracted_matches2.push(ExtractedMatch {
			   kind: m.kind,
			   name: m.name,
			   text: text,
			   start: m.start,
			   end: m.end,
			});
		}
            }
            Ok(Some(ExtractedFile {
                file: path.map(|p| p.to_owned()),
                file_type: self.language.to_string(),
                matches: extracted_matches2,
            }))
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedFile<'query> {
    file: Option<PathBuf>,
    file_type: String,
    matches: Vec<ExtractedMatch<'query>>,
}

impl<'query> Display for ExtractedFile<'query> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: is there a better way to do this unwrapping? This implementation
        // turns non-UTF-8 paths into "NON-UTF8 FILENAME". I don't know exactly
        // what circumstances that could happen in... maybe we should just wait
        // for bug reports?
        let filename = self
            .file
            .as_ref()
            .map(|f| f.to_str().unwrap_or("NON-UTF8 FILENAME"))
            .unwrap_or("NO FILE");

        for extraction in &self.matches {
            writeln!(
                f,
                "{}:{}:{}:{}:{}",
                filename,
                extraction.start.row + 1,
                extraction.start.column + 1,
                extraction.name,
                extraction.text
            )?
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedMatch<'query> {
    kind: &'static str,
    name: &'query str,
    text: String,
    #[serde(serialize_with = "serialize_point")]
    start: Point,
    #[serde(serialize_with = "serialize_point")]
    end: Point,
}

fn serialize_point<S>(point: &Point, sz: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut out = sz.serialize_struct("Point", 2)?;
    out.serialize_field("row", &(point.row + 1))?;
    out.serialize_field("column", &(point.column + 1))?;
    out.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use tree_sitter::Parser;

    #[test]
    fn test_matches_are_extracted() {
        let lang = Language::Elm;
        let query = lang
            .parse_query("(import_clause (upper_case_qid)@import)")
            .unwrap();
        let extractor = Extractor::new(lang, query);

        let extracted = extractor
            .extract_from_text(None, b"import Html.Styled", &mut Parser::new())
            // From Result<Option<ExtractedFile>>
            .unwrap()
            // From Option<ExtractedFile>
            .unwrap();

        assert_eq!(extracted.matches.len(), 1);
        assert_eq!(extracted.matches[0].name, "import");
        assert_eq!(extracted.matches[0].text, "Html.Styled");
    }

    #[test]
    fn test_underscore_names_are_ignored() {
        let lang = Language::Elm;
        let query = lang
            .parse_query("(import_clause (upper_case_qid)@_import)")
            .unwrap();
        let extractor = Extractor::new(lang, query);

        let extracted = extractor
            .extract_from_text(None, b"import Html.Styled", &mut Parser::new())
            // From Result<Option<ExtractedFile>>
            .unwrap();

        assert_eq!(extracted, None);
    }

    #[test]
    fn test_underscore_names_can_still_be_used_in_matchers() {
        let lang = Language::JavaScript;
        let query = lang
            .parse_query("(call_expression (identifier)@_fn (arguments . (string)@import .) (#eq? @_fn require))")
            .unwrap();
        let extractor = Extractor::new(lang, query);

        let extracted = extractor
            .extract_from_text(None, b"let foo = require(\"foo.js\")", &mut Parser::new())
            // From Result<Option<ExtractedFile>>
            .unwrap()
            // From Option<ExtractedFile>
            .unwrap();

        assert_eq!(extracted.matches.len(), 1);
        assert_eq!(extracted.matches[0].name, "import");
        assert_eq!(extracted.matches[0].text, "\"foo.js\"");
    }
}
