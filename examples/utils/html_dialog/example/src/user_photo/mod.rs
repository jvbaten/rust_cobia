use html_dialog::{HtmlDialogHandler,HtmlDialogResourceType,HtmlDialog,Window};

/// Define an enumeration with representing all possible variations
/// that require non-static content to be evaluated. 
///
/// The enumeration value is used for the resource map that maps
/// a URL path to a resource.
///
/// If non-static content is requested, the `provide_content` method
/// is invoked with the enumeration value.

pub(crate) enum UserPhotoDialogEvent {
}

/// The dialog in- and output is arranged by a data handler.
///
/// Define a structure that keeps track of the internal state of 
/// the dialog; that is, all the data members that are needed to
/// provide the dynamic content.

pub(crate) struct UserPhotoDialogHandler {
}

impl UserPhotoDialogHandler {

    /// Create a new instance of the dialog
    ///
    /// The dialog instance is created with a data handler instance
    /// that keeps track of the internal state of the dialog.
    ///
    /// # Returns
    /// An instance of `HtmlDialog` with the data handler specified.

    pub fn new() -> HtmlDialog<UserPhotoDialogHandler,UserPhotoDialogEvent> {
        //define an instance of the dialog data handler    
        let dialog_data_handler=UserPhotoDialogHandler {
        };
        //create a dialog instance, specifying the data handler
        let mut dlg=HtmlDialog::<UserPhotoDialogHandler,UserPhotoDialogEvent>::new(dialog_data_handler);
        //add resources to the dialog
        // - note that static content is linked directly into the binary using include_bytes!
        // - nonstatic content is obtained through the  `provide_content` method, given the enumeration value
        dlg.add("/".into(),HtmlDialogResourceType::Content((include_bytes!("user_photo.html"),"text/html")));
        dlg.add("/user_photo.css".into(),HtmlDialogResourceType::Content((include_bytes!("user_photo.css"),"text/css")));
        dlg.add("/user_photo.js".into(),HtmlDialogResourceType::Content((include_bytes!("user_photo.js"),"text/javascript")));
        dlg.add("/user_photo.png".into(),HtmlDialogResourceType::Content((include_bytes!("user_photo.png"),"image/png")));
        dlg.add("/passfoto.jpg".into(),HtmlDialogResourceType::Content((include_bytes!("passfoto.jpg"),"image/jpg")));
        dlg
    }
}

impl HtmlDialogHandler<UserPhotoDialogEvent> for UserPhotoDialogHandler {

    /// Provide dynamic content for the dialog.
    ///
    /// This method is invoked when the dialog requests non-static content.
    ///
    /// # Arguments
    /// * `event` - The enumeration value that indicates which content is requested.
    /// * `_content` - Optional content that is provided in case of a POST request.
    ///
    /// # Returns
    /// A tuple containing the content as a vector of bytes and the MIME type as a string.
    fn provide_content(&mut self, event: &UserPhotoDialogEvent, _content: Option<String>, _parent:Option<Window>) -> Result<(Vec<u8>,String), Box<dyn std::error::Error>> {
        match event {
            //match the enumeration value and provide the corresponding content
            _ => Err("Invalid dynamic content request".into())
        }
    }
}
