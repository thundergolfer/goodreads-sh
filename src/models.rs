use std::fs;
use std::io::Read;
use std::path::PathBuf;

use roxmltree::{Node, ExpandedName};

struct Shelf {
    books: Vec<Book>
}

struct Book {
    id: i64,
    description: String,
    title: String
}


fn parse_shelf(shelf_xml: &str) -> Result<Shelf, roxmltree::Error> {
    let mut books: Vec<Book> = Vec::new();
    let doc = match roxmltree::Document::parse(shelf_xml) {
        Ok(doc) => doc,
        Err(e) => {
            println!("Error: {}.", e);
            return Err(e);
        },
    };
    
    for node in doc.descendants() {
        if node.is_element() && node.has_tag_name("book") {
            books.push(book_from_xml_node(node));
        }
    }

    Ok(Shelf { books })
}

fn book_from_xml_node(node: Node) -> Book {
    let mut b = Book {
        id: -1,
        description: "".to_owned(),
        title: "".to_owned(),
    };

    for child_node in node.descendants() {
        match child_node.tag_name().name() {
            "id" => {
                b.id = child_node.text().unwrap().parse::<i64>().unwrap();
            }
            "description" => {
                b.description = String::from(child_node.text().unwrap());
            }
            "title" => {
                b.title = String::from(child_node.text().unwrap());
            }
            _ => {}
        }
    }
    b
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