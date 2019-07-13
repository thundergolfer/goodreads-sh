use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::path::PathBuf;
use std::fs;
use regex::Regex;

use roxmltree::Node;

const MAX_DESC_LEN: usize = 20;

pub struct Shelf {
    pub books: Vec<Book>,
}

pub struct Book {
    pub id: u32,
    pub description: String,
    pub title: String,
    // Sometimes num_pages is missing from XML data.
    pub num_pages: Option<u32>,
}

impl Display for Book {
    // `f` is a buffer, this method must write the formatted string into it
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let shortened_desc = &self.description[..MAX_DESC_LEN];

        // `write!` is like `format!`, but it will write the formatted string
        // into a buffer (the first argument)
        write!(f, "{}: {}...", self.title, shortened_desc)
    }
}

pub fn parse_shelf(shelf_xml: &str) -> Result<Shelf, roxmltree::Error> {
    let mut books: Vec<Book> = Vec::new();
    let doc = match roxmltree::Document::parse(shelf_xml) {
        Ok(doc) => doc,
        Err(e) => {
            println!("Error: {}.", e);
            return Err(e);
        }
    };

    for node in doc.descendants() {
        if node.is_element() && node.has_tag_name("book") {
            books.push(book_from_xml_node(node));
        }
    }

    Ok(Shelf { books })
}

/// For some insane reason the 'id' field that appears in the XML is NOT
/// the id value that you want to use in API calls.
/// The usable ID value can only be found in URLs in the XML object.
/// This function can extract the ID from the '<link>' node.
fn extract_book_id_from_book_link(book_link: &str) -> Option<u32> {
    let re: Regex = Regex::new(r"(?x)
        ^https://www.goodreads.com/book/show/(?P<book_id>[\d]+)[\-|\.]
        ([[:word:]]+\-)*
        ([[:word:]]+_)*
        [[:word:]]+$
        ").unwrap();
    re.captures(book_link).and_then(|cap| {
        cap.name("book_id")
            .map(|book_id| book_id.as_str())
            .map(|book_id| book_id.parse::<u32>().unwrap())
    })
}

fn book_from_xml_node(node: Node) -> Book {
    let mut book = Book {
        id: 0,
        description: "".to_owned(),
        title: "".to_owned(),
        num_pages: None,
    };

    for child_node in node.descendants() {
        match child_node.tag_name().name() {
            "link" => {
                let parent = child_node.parent();
                // Don't attempt to parse the <link> node that is within the <author> node.
                if parent.is_some() && parent.unwrap().tag_name().name() == "book" {
                    let link_txt = child_node.text().unwrap();
                    let book_id = extract_book_id_from_book_link(link_txt);
                    book.id = book_id.expect(
                        &format!("Could not get book id from <link> URL: {}", link_txt)
                    );
                }
            }
            "description" => {
                book.description = String::from(child_node.text().unwrap());
            }
            "title" => {
                book.title = String::from(child_node.text().unwrap());
            }
            "num_pages" => {
                book.num_pages = match child_node.text() {
                    Some(val) => Some(val.parse::<u32>().unwrap()),
                    _ => None,
                }
            }
            _ => {}
        }
    }
    book
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_file(path: &PathBuf) -> String {
        let mut file = fs::File::open(&path).unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        text
    }

    #[test]
    fn test_parse_shelf() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/api_responses/currently_reading_shelf_resp.xml");
        println!("{}", path.display());
        let text = load_file(&path);
        parse_shelf(&text);
    }
}
