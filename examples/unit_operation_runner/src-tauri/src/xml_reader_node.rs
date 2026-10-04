/// Streaming XML reader wrapper used by persistence deserialization.
///
/// Owns a `quick_xml` reader and tracks status of reading nodes, node content, attributes, and end-of-file.
///
/// Provides a cursor-like `XMLNode` handle that can be used to navigate and read individual XML nodes in sequential order.
pub(crate) struct XMLReader {
    /// Underlying pull-based XML reader over the input file.
    xml_reader: quick_xml::reader::Reader<std::io::BufReader<std::fs::File>>,
    /// `true` once end-of-file has been encountered.
    end: bool,
}

impl XMLReader {
    /// Opens an XML file and creates a shared reader state.
    ///
    /// # Parameters
    /// - `path`: Filesystem path to the XML document.
    ///
    /// # Returns
    /// - `Ok(Rc<RefCell<XMLReader>>)` when the file is opened successfully.
    /// - `Err(e)` if the file cannot be opened.
    fn new(
        path: std::path::PathBuf,
    ) -> Result<std::rc::Rc<std::cell::RefCell<Self>>, Box<dyn std::error::Error>> {
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Error opening file: {}", e).into());
            }
        };
        let reader = std::io::BufReader::new(file);
        let xml_reader = quick_xml::reader::Reader::from_reader(reader);
        Ok(std::rc::Rc::new(std::cell::RefCell::new(Self {
            xml_reader,
            end: false,
        })))
    }
}

/// Cursor-like handle over one XML node in the stream.
///
/// `XMLNode` keeps a shared reference to the underlying [`XMLReader`] and
/// allows reading node text and iterating over child nodes.
pub(crate) struct XMLNode {
    /// Shared reader state backing this cursor.
    reader: std::rc::Rc<std::cell::RefCell<XMLReader>>,
    /// Element name.
    name: String,
    /// Decoded `(key, value)` attribute pairs from the element start tag.
    attributes: Vec<(String, String)>,
    /// Scratch buffer reused for pull parsing events.
    buf: Vec<u8>,
    /// `true` when this node's end tag has been consumed.
    end_read: bool,
}

impl XMLReader {
    /// Opens an XML file and returns the first root/start node, if any.
    ///
    /// This method skips non-start events until it finds `Start`/`Empty`, EOF,
    /// or an error.
    ///
    /// # Parameters
    /// - `path`: Filesystem path to the XML document.
    ///
    /// # Returns
    /// - `Ok(Some(XMLNode))` when the first element is found.
    /// - `Ok(None)` when the document is empty (EOF before any start tag).
    /// - `Err(e)` on XML read/attribute decode errors or unexpected end tags.
    pub fn root(path: std::path::PathBuf) -> Result<Option<XMLNode>, Box<dyn std::error::Error>> {
        let reader = Self::new(path)?;
        let mut buf = Vec::new();
        loop {
            let event = reader.borrow_mut().xml_reader.read_event_into(&mut buf);
            match &event {
                Err(e) => {
                    return Err(format!("Error reading XML position: {}", e).into());
                }
                // exits the loop when reaching end of file
                Ok(quick_xml::events::Event::Eof) => {
                    reader.borrow_mut().end = true;
                    return Ok(None);
                }
                Ok(quick_xml::events::Event::Start(e)) | Ok(quick_xml::events::Event::Empty(e)) => {
                    let mut end_read = false;
                    if let Ok(quick_xml::events::Event::Empty(_)) = event {
                        end_read = true;
                    };
                    let mut attributes = Vec::new();
                    for attr in e.attributes() {
                        match attr {
                            Err(e) => {
                                return Err(format!("Error reading XML attribute: {}", e).into());
                            }
                            Ok(a) => {
                                let key = a.key.as_ref().to_string();
                                let value = a
                                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                    .map_err(|e| {
                                        format!("Error reading XML attribute value: {}", e)
                                    })?
                                    .to_string();
                                attributes.push((key, value));
                            }
                        }
                    }
                    return Ok(Some(XMLNode {
                        reader: reader.clone(),
                        name: e.name().as_ref().to_string(),
                        buf: Vec::new(),
                        end_read,
                        attributes,
                    }));
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    //unexpected
                    return Err(format!(
                        "Unexpected end in xml: {}",
                        e.name().as_ref()
                    )
                    .into());
                }
                _ => {}
            }
        }
    }
}

impl XMLNode {
    /// Returns the raw element name bytes for this node.
    ///
    /// # Returns
    /// Slice containing the element name as UTF-8 bytes from the XML stream.
    pub fn get_name(&self) -> &[u8] {
        self.name.as_bytes()
    }

    /// Returns all decoded attributes of this node.
    ///
    /// # Returns
    /// Reference to `(key, value)` pairs parsed from the start tag.
    pub fn get_attributes(&self) -> &Vec<(String, String)> {
        &self.attributes
    }

    /// Reads and returns this node's text content.
    ///
    /// Consumes events until this node's closing tag is reached. XML general
    /// references such as `&lt;` and `&amp;` are resolved into characters.
    ///
    /// # Returns
    /// - `Ok(String)` containing decoded text (possibly empty).
    /// - `Err(e)` on malformed XML, decode errors, or unexpected closing tags.
    pub fn get_text(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.end_read {
            return Ok(String::new());
        }
        let mut text = String::new();
        loop {
            self.buf.clear();
            let event=self
                .reader
                .borrow_mut()
                .xml_reader
                .read_event_into(&mut self.buf);
            match event {
                Err(e) => {
                    return Err(format!("Error reading XML position: {}", e).into());
                }
                // exits the loop when reaching end of file
                Ok(quick_xml::events::Event::Eof) => {
                    self.reader.borrow_mut().end = true;
                    return Ok(text);
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    text.push_str(&e.xml10_content());
                }
                Ok(quick_xml::events::Event::GeneralRef(e)) => {
                    match e.into_inner().as_ref() {
                        "lt" => text.push('<'),
                        "gt" => text.push('>'),
                        "amp" => text.push('&'),
                        "apos" => text.push('\''),
                        "quot" => text.push('"'),
                        _ => {}
                    };
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    if e.name().as_ref() == self.name {
                        self.end_read = true;
                        return Ok(text);
                    }
                    //unexpected
                    return Err(format!(
                        "Unexpected end in xml: {}",
                        e.name().as_ref()
                    )
                    .into());
                }
                _ => {}
            }
        }
    }

    /// Advances to the next child node within this node.
    ///
    /// # Returns
    /// - `Ok(Some(XMLNode))` when a child start/empty tag is found.
    /// - `Ok(None)` when this node has no more children or reaches its end tag.
    /// - `Err(e)` on malformed XML, attribute decode errors, or unexpected tags.
    pub fn next_node(&mut self) -> Result<Option<XMLNode>, Box<dyn std::error::Error>> {
        if self.end_read {
            return Ok(None);
        }
        loop {
            self.buf.clear();
            let event = self
                .reader
                .borrow_mut()
                .xml_reader
                .read_event_into(&mut self.buf);
            match &event {
                Err(e) => {
                    return Err(format!("Error reading XML position: {}", e).into());
                }
                // exits the loop when reaching end of file
                Ok(quick_xml::events::Event::Eof) => {
                    self.reader.borrow_mut().end = true;
                    return Ok(None);
                }
                Ok(quick_xml::events::Event::Start(e)) | Ok(quick_xml::events::Event::Empty(e)) => {
                    let mut end_read = false;
                    if let Ok(quick_xml::events::Event::Empty(_)) = event {
                        end_read = true;
                    };
                    let mut attributes = Vec::new();
                    for attr in e.attributes() {
                        match attr {
                            Err(e) => {
                                return Err(format!("Error reading XML attribute: {}", e).into());
                            }
                            Ok(a) => {
                                let key = a.key.as_ref().to_string();
                                let value = a
                                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                    .map_err(|e| {
                                        format!("Error reading XML attribute value: {}", e)
                                    })?
                                    .to_string();
                                attributes.push((key, value));
                            }
                        }
                    }
                    return Ok(Some(XMLNode {
                        reader: self.reader.clone(),
                        name: e.name().as_ref().to_string(),
                        buf: Vec::new(),
                        end_read,
                        attributes,
                    }));
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    if e.name().as_ref() == self.name {
                        self.end_read = true;
                        return Ok(None);
                    }
                    //unexpected
                    return Err(format!(
                        "Unexpected end in xml: {}",
                        e.name().as_ref()
                    )
                    .into());
                }
                _ => {}
            }
        }
    }
}

impl Drop for XMLNode {
    /// Ensures the parser is advanced to this node's end before dropping.
    ///
    /// This keeps the shared streaming reader aligned so sibling nodes can be
    /// read correctly even if callers stop consuming this node early.
    fn drop(&mut self) {
        if !self.end_read {
            //pul unused nodes
            loop {
                match self.next_node() {
                    Err(_) => {}
                    Ok(None) => break,
                    Ok(Some(_)) => {
                        //ignore
                    }
                }
            }
            if !self.end_read {
                //read until end of node
                loop {
                    self.buf.clear();
                    let event=self
                        .reader
                        .borrow_mut()
                        .xml_reader
                        .read_event_into(&mut self.buf);
                    match event {
                        Err(_) => {}
                        Ok(quick_xml::events::Event::Eof) => {
                            self.reader.borrow_mut().end = true;
                            break;
                        }
                        Ok(quick_xml::events::Event::Start(_))
                        | Ok(quick_xml::events::Event::Empty(_)) => {
                            //ignore - unreachable for succesfull parse
                        }
                        Ok(quick_xml::events::Event::End(e))
                            if e.name().as_ref() == self.name => {
                                break;
                            }
                            //ignore - unreachable for succesfull parse
                        _ => {}
                    }
                }
            }
        }
    }
}
