mod imp {
    use gio::{Menu, MenuItem};
    use gtk::{gio, CompositeTemplate, TemplateChild};

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(string = "
        template FilterMenu: Menu {
            CheckMenuItem toggle_filter {}
        }
    ")]
    pub struct FilterMenu {
        #[template_child]
        toggle_filter: TemplateChild<CheckMenuItem>,
    }
}
