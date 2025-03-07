use relm4::gtk;

#[derive(Debug, Default)]
struct Progress {
    bar: gtk::ProgressBar,
    items: Option<u64>,
}

impl Progress {
    fn new(items: Option<u64>, text: &str) -> Self {
        let bar = gtk::ProgressBarBuilder::new()
            .fraction(0.)
            .show_text(true)
            .text(text)
            .build();
        Progress { bar, items }
    }

    fn set_items(&mut self, items: Option<u64>) {
        self.items = items;
    }

    fn get_bar(&self) -> gtk::ProgressBar {
        // Because this is a gtk object, this clone is a reference to the original
        self.bar.clone()
    }

    fn tick(&self) {
        match self.items {
            Some(i) => self.bar.set_fraction(self.bar.get_fraction() + 1. / i),
            None => bar.pulse(),
        }
    }
}
