use serde::Serialize;

#[derive(Serialize, Clone)]
/// A single cell in a status table row sent to the frontend.
pub(crate) struct TableCell {
    /// Display text shown for this cell.
    pub value: String,
    /// Optional edit target identifier used by the UI when this cell is editable.
    pub edit_id: Option<String>,
    /// Optional list of selectable values for this cell.
    pub options: Option<Vec<String>>,
    /// Optional flag indicating whether options are mutually exclusive.
    pub exclusive: Option<bool>,
}

/// A row in a status table.
///
/// Rows can represent either section headers or ordinary data rows.
#[derive(Serialize, Clone)]
pub(crate) struct TableRow {
    /// Stable row identifier used for UI reconciliation and tree structure.
    pub id: String,
    /// `true` for header rows, `false` for regular data rows.
    pub is_header: bool,
    /// Cells contained in this row. Header rows created with [`TableRow::header`] may have an empty list.
    pub cells: Vec<TableCell>, //for a header may be empty, then id is text
    /// Optional parent row ID for hierarchical table rendering.
    pub parent_id: Option<String>,
}

/// Serializable snapshot of a complete table status payload.
#[derive(Serialize, Clone)]
pub(crate) struct TableStatus {
    /// Ordered column headers.
    pub headers: Vec<String>,
    /// Ordered table rows.
    pub rows: Vec<TableRow>,
}

impl TableRow {
    /// Creates a header row without cells.
    ///
    /// # Parameters
    /// - `id`: Row identifier (also used as display text for empty-header rows).
    /// - `parent`: Optional parent row reference used to build hierarchy.
    ///
    /// # Returns
    /// A header [`TableRow`] with no cells.
    pub fn header(id: &str, parent: Option<&TableRow>) -> Self {
        TableRow {
            id: id.to_string(),
            is_header: true,
            cells: Vec::new(),
            parent_id: parent.map(|p| p.id.clone()),
        }
    }

    /// Creates a header row containing one text cell.
    ///
    /// # Parameters
    /// - `id`: Row identifier.
    /// - `text`: Header text stored in the first cell.
    /// - `parent`: Optional parent row reference used to build hierarchy.
    ///
    /// # Returns
    /// A header [`TableRow`] with a single non-editable cell.
    pub fn new_header(id: String, text: &str, parent: Option<&TableRow>) -> Self {
        let cells = vec![TableCell {
            value: text.to_string(),
            edit_id: None,
            options: None,
            exclusive: None,
        }];
        TableRow {
            id,
            is_header: true,
            cells,
            parent_id: parent.map(|p| p.id.clone()),
        }
    }

    /// Creates an empty non-header data row.
    ///
    /// # Parameters
    /// - `id`: Row identifier.
    /// - `parent`: Optional parent row reference used to build hierarchy.
    ///
    /// # Returns
    /// A data [`TableRow`] with no cells.
    pub fn data_row(id: String, parent: Option<&TableRow>) -> Self {
        Self {
            id,
            is_header: false,
            cells: Vec::new(),
            parent_id: parent.map(|p| p.id.clone()),
        }
    }

    /// Appends a plain, non-editable text cell.
    ///
    /// # Parameters
    /// - `value`: Display text for the new cell.
    pub fn add_cell(&mut self, value: &str) {
        let cell = TableCell {
            value: value.to_string(),
            edit_id: None,
            options: None,
            exclusive: None,
        };
        self.cells.push(cell);
    }

    /// Appends an editable field cell.
    ///
    /// # Parameters
    /// - `value`: Display text/value for the new cell.
    /// - `edit_id`: Optional edit target identifier consumed by the UI.
    pub fn add_field(&mut self, value: String, edit_id: Option<String>) {
        let cell = TableCell {
            value,
            edit_id,
            options: None,
            exclusive: None,
        };
        self.cells.push(cell);
    }

    /// Appends a cell with optional selectable options.
    ///
    /// # Parameters
    /// - `value`: Display text/value for the new cell.
    /// - `edit_id`: Optional edit target identifier consumed by the UI.
    /// - `options`: Optional list of choices shown by the UI.
    /// - `exclusive`: Optional exclusivity flag for choice handling.
    pub fn with_options(
        &mut self,
        value: String,
        edit_id: Option<String>,
        options: Option<Vec<String>>,
        exclusive: Option<bool>,
    ) {
        let cell = TableCell {
            value,
            edit_id,
            options,
            exclusive,
        };
        self.cells.push(cell);
    }

    /// Appends a non-editable result cell.
    ///
    /// # Parameters
    /// - `value`: Result text/value for the new cell.
    pub fn add_result(&mut self, value: String) {
        let cell = TableCell {
            value,
            edit_id: None,
            options: None,
            exclusive: None,
        };
        self.cells.push(cell);
    }

    /// Appends an empty cell.
    pub fn add_empty(&mut self) {
        let cell = TableCell {
            value: String::new(),
            edit_id: None,
            options: None,
            exclusive: None,
        };
        self.cells.push(cell);
    }
}
