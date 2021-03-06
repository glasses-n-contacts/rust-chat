extern crate http_muncher;
use self::http_muncher::ParserHandler;
use std::str;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HttpParser {
    pub current_key: Option<String>,
    pub headers: Rc<RefCell<HashMap<String, String>>>
}

impl ParserHandler for HttpParser {
    fn on_header_field(&mut self, s: &[u8]) -> bool {
        self.current_key = Some(str::from_utf8(s).unwrap().to_string());
        true
    }

    fn on_header_value(&mut self, s: &[u8]) -> bool {
        self.headers.borrow_mut()
            .insert(self.current_key.clone().unwrap(),
                    str::from_utf8(s).unwrap().to_string());
        true
    }

    fn on_headers_complete(&mut self) -> bool {
        false
    }
}