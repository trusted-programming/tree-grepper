use crate::language::Language;
use anyhow::{Context, Result};
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Point, QueryCursor};

#[derive(Debug)]
pub struct Extractor {
    language: Language,
    ts_language: tree_sitter::Language,
}

impl Extractor {
    pub fn new(language: Language) -> Extractor {
        Extractor {
            ts_language: (&language).language(),
            language,
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
        let sources = splitup_unsafe(parser, self.ts_language, &source).ok().unwrap();
        for (file, m) in sources.iter() {
            let src = *m;
            let digest = md5::compute(&src);
            let file_name = path
                .parent()
                .unwrap()
                .join(path.file_stem().unwrap())
                .join(format!("{:x}.rs.unsafe", digest));
            if !file_name.parent().unwrap().exists() {
                std::fs::create_dir(&file_name.parent().unwrap())?;
            }
            std::fs::write(&file_name, std::str::from_utf8(&src).unwrap())?;
        }
        let sources = splitup_all(parser, self.ts_language, &source).ok().unwrap();
        for (file, m) in sources.iter() {
            let src = *m;
            let digest = md5::compute(&src);
            let unsafe_file_name = path
                .parent()
                .unwrap()
                .join(path.file_stem().unwrap())
                .join(format!("{:x}.rs.unsafe", digest));
            if ! unsafe_file_name.exists() {
                let file_name = path
                    .parent()
                    .unwrap()
                    .join(path.file_stem().unwrap())
                    .join(format!("{:x}.rs.safe", digest));
                if !file_name.parent().unwrap().exists() {
                    std::fs::create_dir(&file_name.parent().unwrap())?;
                }
                std::fs::write(&file_name, std::str::from_utf8(&src).unwrap())?;
            }
        }
 
        Ok(None)
    }
}

// Split source code by blocks
//
fn splitup_unsafe<'a>(
    parser: &mut Parser,
    ts_language: tree_sitter::Language,
    source: &'a [u8],
) -> Result<HashMap<usize, &'a [u8]>> {
    parser
        .set_language(ts_language)
        .context("could not set language")?;
    let tree = parser
        .parse(&source, None)
        .context("could not parse to a tree. This is an internal error and should be reported.")?;
    let query = Language::Rust
        .parse_query(
            "[(function_item (function_modifiers) body: (block) @ub) (unsafe_block (block)@ub)]",
        )
        .unwrap();
    let captures = query.capture_names().to_vec();
    let mut cursor = QueryCursor::new();
    let extracted = cursor
        .matches(&query, tree.root_node(), source)
        .flat_map(|query_match| query_match.captures)
        .map(|capture| {
            let name = &captures[capture.index as usize];
            let node = capture.node;
            Ok(ExtractedNode {
                name: name,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            })
        })
        .collect::<Result<Vec<ExtractedNode>>>()?;
    let mut output: HashMap<usize, &[u8]> = HashMap::new();
    for m in extracted {
        if m.name == "ub" {
            let code = std::str::from_utf8(&source[m.start_byte..m.end_byte]).unwrap();
            output.insert(m.start_byte, code.as_bytes());
        } else if m.name == "b" {
            let code = std::str::from_utf8(&source[m.start_byte..m.end_byte]).unwrap();
            output.insert(m.start_byte, code.as_bytes());
        }
    }
    Ok(output)
}



fn splitup_all<'a>(
    parser: &mut Parser,
    ts_language: tree_sitter::Language,
    source: &'a [u8],
) -> Result<HashMap<usize, &'a [u8]>> {
    parser
        .set_language(ts_language)
        .context("could not set language")?;
    let tree = parser
        .parse(&source, None)
        .context("could not parse to a tree. This is an internal error and should be reported.")?;
    let query = Language::Rust
        .parse_query(
            "[ ((block)@b) ]",
        )
        .unwrap();
    let captures = query.capture_names().to_vec();
    let mut cursor = QueryCursor::new();
    let extracted = cursor
        .matches(&query, tree.root_node(), source)
        .flat_map(|query_match| query_match.captures)
        .map(|capture| {
            let name = &captures[capture.index as usize];
            let node = capture.node;
            Ok(ExtractedNode {
                name: name,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
            })
        })
        .collect::<Result<Vec<ExtractedNode>>>()?;
    let mut output: HashMap<usize, &[u8]> = HashMap::new();
    for m in extracted {
        if m.name == "ub" {
            let code = std::str::from_utf8(&source[m.start_byte..m.end_byte]).unwrap();
            output.insert(m.start_byte, code.as_bytes());
        } else if m.name == "b" {
            let code = std::str::from_utf8(&source[m.start_byte..m.end_byte]).unwrap();
            output.insert(m.start_byte, code.as_bytes());
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
pub struct ExtractedNode<'query> {
    name: &'query str,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker() {
        let mut parser = Parser::new();
        let language = Language::Rust.language();
    }
}
