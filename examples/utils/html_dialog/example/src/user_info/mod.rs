use html_dialog::{HtmlDialogHandler,HtmlDialogResourceType,HtmlDialog,Window};
use chrono::Local;
use super::user_photo;


/// Define an enumeration with representing all possible variations
/// that require non-static content to be evaluated. 
///
/// The enumeration value is used for the resource map that maps
/// a URL path to a resource.
///
/// If non-static content is requested, the `provide_content` method
/// is invoked with the enumeration value.

pub(crate) enum UserInfoDialogEvent {
    GetUserInfo,
    ShowUserPhoto
}

/// The dialog in- and output is arranged by a data handler.
///
/// Define a structure that keeps track of the internal state of 
/// the dialog; that is, all the data members that are needed to
/// provide the dynamic content.

pub(crate) struct UserInfoDialogHandler {
    number_of_request:i32,
}

impl UserInfoDialogHandler {

    /// Create a new instance of the dialog
    ///
    /// The dialog instance is created with a data handler instance
    /// that keeps track of the internal state of the dialog.
    ///
    /// # Returns
    /// An instance of `HtmlDialog` with the data handler specified.

    pub fn new() -> HtmlDialog<UserInfoDialogHandler,UserInfoDialogEvent> {
        //define an instance of the dialog data handler    
        let dialog_data_handler=UserInfoDialogHandler {
            number_of_request:0
        };
        //create a dialog instance, specifying the data handler
        let mut dlg=HtmlDialog::<UserInfoDialogHandler,UserInfoDialogEvent>::new(dialog_data_handler);
        //add resources to the dialog
        // - note that static content is linked directly into the binary using include_bytes!
        // - nonstatic content is obtained through the  `provide_content` method, given the enumeration value
        dlg.add("/".into(),HtmlDialogResourceType::Content((include_bytes!("user_info.html"),"text/html")));
        dlg.add("/user_info.css".into(),HtmlDialogResourceType::Content((include_bytes!("user_info.css"),"text/css")));
        dlg.add("/user_info.js".into(),HtmlDialogResourceType::Content((include_bytes!("user_info.js"),"text/javascript")));
        dlg.add("/user_info.png".into(),HtmlDialogResourceType::Content((include_bytes!("user_info.png"),"image/png")));
        dlg.add("/get_user_info".into(),HtmlDialogResourceType::Info(UserInfoDialogEvent::GetUserInfo));
        dlg.add("/show_user_photo".into(),HtmlDialogResourceType::Info(UserInfoDialogEvent::ShowUserPhoto));
        dlg
    }
}

impl HtmlDialogHandler<UserInfoDialogEvent> for UserInfoDialogHandler {

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
    fn provide_content(&mut self, event: &UserInfoDialogEvent, _content: Option<String>,parent:Option<Window>) -> Result<(Vec<u8>,String), Box<dyn std::error::Error>> {
        match event {
            UserInfoDialogEvent::GetUserInfo => {
                self.number_of_request+=1;
                let date = Local::now();
                let now =format!("{}",date.format("%d-%m-%Y %H:%M:%S"));
                let user_info=json::object!{
                    full_name: whoami::realname(),
                    user_name: whoami::username(),
                    preferred_language: std::env::var("LANG").unwrap_or("unknown".to_string()),
                    pretty_name: whoami::distro(),
                    desktop_environment: whoami::desktop_env().to_string(),
                    os_name: whoami::distro().to_string(),
                    platform_name: whoami::platform().to_string(),
                    cpu_architecture: whoami::arch().to_string(),
                    number_of_times_asked: self.number_of_request,
                    last_time_asked: now,
                };
                Ok((json::stringify(user_info).into_bytes(),"application/json".to_string()))
            },
            UserInfoDialogEvent::ShowUserPhoto => {
                //show a sub-dialog with a fictive photo of the user
                let mut user_photo_dlg=user_photo::UserPhotoDialogHandler::new();
                user_photo_dlg.set_parent(parent,true);
                user_photo_dlg.show()?;
                Ok((Vec::new(),"application/text".to_string()))
            }
        }
    }
}
