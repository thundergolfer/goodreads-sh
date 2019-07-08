use std::fs;
use std::io::Read;
use std::path::PathBuf;

struct Shelf {
    books: Vec<Book>
}

struct Book {
    id: i64,
    description: String,
    title: String
}


fn parse_shelf(shelf_xml: &str) {
    let doc = match roxmltree::Document::parse(shelf_xml) {
        Ok(doc) => doc,
        Err(e) => {
            println!("Error: {}.", e);
            return;
        },
    };

    // TODO: finish
    for node in doc.descendants() {
        if node.is_element() {
            println!("{:?} at {}", node.tag_name(), doc.text_pos_at(node.range().start));
        }
    }
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