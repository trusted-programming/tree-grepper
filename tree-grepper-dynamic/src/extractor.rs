use crate::language::Language;
use anyhow::{Context, Result};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Point, Query, QueryCursor};

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
        let source_without_comment = remove_comment(parser, self.ts_language, &source)?;
        let source_with_unsafe = markup_unsafe(parser, self.ts_language, &source_without_comment)?;
        println!("{}", std::str::from_utf8(&source_without_comment).unwrap());
        println!("=======");
        println!("{}", std::str::from_utf8(&source_with_unsafe).unwrap());
        Ok(None)
        // self.extract_from_text(Some(path), &source_without_comment, parser)
    }
}

fn markup_unsafe(parser: &mut Parser, ts_language: tree_sitter::Language, source: &[u8]) -> Result<Vec<u8>> {
    parser
        .set_language(ts_language)
        .context("could not set language")?;
    let mut tree = parser
        .parse(&source, None)
        .context("could not parse to a tree. This is an internal error and should be reported.")?;
    let query = Language::Rust
        .parse_query("([
            (_ safeness: _) @safe 
            (function_item ! function_modifiers) @safe 
            (unsafe_block) @ub      
            (safeness) @uf 
        ]) ")
        .unwrap();
    let mut cursor = QueryCursor::new();
    let extracted = cursor
        .matches(&query, tree.root_node(), source)
        .flat_map(|query_match| query_match.captures)
        .map(|capture| {
            let node = capture.node;
            let text = match node
                .utf8_text(source)
                .map(|unowned| unowned.to_string())
                .context("could not extract text from capture")
            {
                Ok(text) => text,
                Err(problem) => return Err(problem),
            };
            Ok(ExtractedComment {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            })
        })
        .collect::<Result<Vec<ExtractedComment>>>()?;
    let mut output = Vec::new();
    for (i, c) in source.iter().enumerate() {
        let mut found = false;
        for m in &extracted {
            if m.start_byte <= i && i < m.end_byte {
                found = true;
                if (i == m.start_byte) {
                    output.extend("<SAFENESS>unsafe</SAFENESS>".to_string().as_bytes());
                }
            }
        }
        let buf: &mut [u8] = &mut [0];
        if !found {
            output.push(*c);
        }
    }
    Ok(output)
}

// Remove all comments in Rust code by checking whether a character is within the specified range of a line or block comment
// These comment nodes are obtained from the query: "([(line_comment)(block_comment)])@c")
fn remove_comment(
    parser: &mut Parser,
    ts_language: tree_sitter::Language,
    source: &[u8],
) -> Result<Vec<u8>> {
    parser
        .set_language(ts_language)
        .context("could not set language")?;
    let mut tree = parser
        .parse(&source, None)
        .context("could not parse to a tree. This is an internal error and should be reported.")?;
    let query = Language::Rust
        .parse_query("([(line_comment)(block_comment)])@c")
        .unwrap();
    let mut cursor = QueryCursor::new();
    let extracted = cursor
        .matches(&query, tree.root_node(), source)
        .flat_map(|query_match| query_match.captures)
        .map(|capture| {
            let node = capture.node;
            let text = match node
                .utf8_text(source)
                .map(|unowned| unowned.to_string())
                .context("could not extract text from capture")
            {
                Ok(text) => text,
                Err(problem) => return Err(problem),
            };
            Ok(ExtractedComment {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            })
        })
        .collect::<Result<Vec<ExtractedComment>>>()?;
    let mut output = Vec::new();
    for (i, c) in source.iter().enumerate() {
        let mut found = false;
        for m in &extracted {
            if m.start_byte <= i && i < m.end_byte {
                found = true;
            }
        }
        let buf: &mut [u8] = &mut [0];
        if !found {
            output.push(*c);
        }
    }
    Ok(output)
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedFile<'query> {
    file: Option<PathBuf>,
    file_type: String,
    matches: Vec<ExtractedMatch<'query>>,
}

impl<'query> Display for ExtractedFile<'query> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let filename = self
            .file
            .as_ref()
            .map(|f| f.to_str().unwrap_or("NON-UTF8 FILENAME"))
            .unwrap_or("NO FILE");

        for extraction in &self.matches {
            writeln!(
                f,
                "{}:{}:{}:{}:{}:{}:{}:{}:{}",
                filename,
                extraction.start_byte,
                extraction.start.row + 1,
                extraction.start.column + 1,
                extraction.end_byte,
                extraction.end.row + 1,
                extraction.end.column + 1,
                extraction.name,
                extraction.text
            )?
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedComment {
    start_byte: usize,
    end_byte: usize,
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtractedMatch<'query> {
    kind: &'static str,
    name: &'query str,
    text: String,
    start_byte: usize,
    end_byte: usize,
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
