use gtk::prelude::*;
use relm4::adw::Window;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::drawing::{DrawContext, DrawHandler};
use relm4::gtk::cairo::Operator;
use relm4::gtk::gdk_pixbuf::{InterpType, Pixbuf};
use relm4::gtk::prelude::GdkCairoContextExt;
use relm4::gtk::{PrintOperation, PrintSettings};
use relm4::{gtk, AsyncComponentSender};

use super::ViewPreviewMsg;

#[derive(Debug, Default)]
pub struct ImageWidget {
    pub original: Option<Pixbuf>,
    pub preview: Option<Pixbuf>,
    view_width: u32,
    view_height: u32,
    zoom: u32,
    handler: DrawHandler,
}

#[derive(Debug)]
pub enum ImageMsg {
    Resize((u32, u32)),
    Scale(Option<u32>),
    SetImage(Option<Pixbuf>),
    UpdatePreview,
    Print(Window),
}

#[relm4::component(async, pub)]
impl AsyncComponent for ImageWidget {
    type Init = ();
    type Input = ImageMsg;
    type Output = ViewPreviewMsg;
    type CommandOutput = ();

    view! {
        gtk::Box {
            #[local_ref]
            area -> gtk::DrawingArea {
                set_vexpand: true,
                set_hexpand: true,

                connect_resize[sender] => move |_, x, y| {
                    tracing::info!("Resizing to {x} x {y}");
                    sender.input(ImageMsg::Resize((x as u32, y as u32)));
                }

            }
        }
    }

    async fn init(
        _: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = ImageWidget {
            ..Default::default()
        };
        let area = model.handler.drawing_area();

        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    #[tracing::instrument(
        name = "Updating Image Preview",
        level = "info",
        skip(self, sender, _root)
    )]
    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ImageMsg::UpdatePreview => {
                let cx: DrawContext = self.handler.get_context();
                cx.save().unwrap();
                cx.set_operator(Operator::Clear);
                cx.paint().unwrap();
                cx.restore().unwrap();

                if let Some(preview) = &self.preview {
                    tracing::info!("Setting preview pixbuf");
                    let pos_x = (self.view_width as i32 - preview.width()) / 2;
                    let pos_y = (self.view_height as i32 - preview.height()) / 2;
                    cx.set_source_pixbuf(preview, pos_x as f64, pos_y as f64);
                } else {
                    cx.set_source_rgba(0., 0., 0., 0.);
                }
                cx.paint().expect("Couldn't fill context");
            }
            ImageMsg::Resize((x, y)) => {
                self.view_width = x;
                self.view_height = y;
                if self.original.is_some() {
                    self.preview.replace(self.scale_to_fit(x, y).unwrap());
                }
                sender.input(ImageMsg::UpdatePreview);
            }
            ImageMsg::Scale(scale) => {
                if let Some(s) = scale {
                    self.view_scale(s);
                } else {
                    // Resize to fit in the available space
                    self.preview.replace(
                        self.scale_to_fit(self.view_width, self.view_height)
                            .unwrap(),
                    );
                }
                sender.input(ImageMsg::UpdatePreview);
            }
            ImageMsg::SetImage(image) => {
                self.original = image;
                self.update_preview();
                sender.input(ImageMsg::UpdatePreview);
            }
            ImageMsg::Print(window) => self.print(&window),
        }
    }
}

impl ImageWidget {
    #[tracing::instrument(name = "Updating Preview from Original", level = "info", skip(self))]
    pub fn update_preview(&mut self) {
        if self.view_width == 0 && self.view_height == 0 {
            self.preview = self.original.clone();
        } else if self.original.is_some() {
            tracing::info!("Resizing to {} x {}.", self.view_width, self.view_height);
            self.preview.replace(
                self.scale_to_fit(self.view_width, self.view_height)
                    .expect("Unable to resize image"),
            );
        }
    }

    pub fn view_scale(&mut self, scale: u32) {
        tracing::info!("Zooming image to {scale:?}.");
        if let Some(i) = &self.original {
            self.preview.replace(
                i.scale_simple(
                    (i.width() as f64 * (scale as f64 / 100.0)) as i32,
                    (i.height() as f64 * (scale as f64 / 100.0)) as i32,
                    InterpType::Bilinear,
                )
                .unwrap(),
            );
        };
    }

    pub fn scale_to_fit(&self, width: u32, height: u32) -> Option<Pixbuf> {
        if let Some(image) = &self.original {
            let image_width = image.width() as f64;
            let image_height = image.height() as f64;
            let width_ratio = width as f64 / image_width;
            let height_ratio = height as f64 / image_height;
            let scale_ratio = width_ratio.min(height_ratio);
            // Perform the calculations for the scale in f64 to
            // remove as much of the rounding error as possible.
            image.scale_simple(
                (image_width * scale_ratio) as i32,
                (image_height * scale_ratio) as i32,
                InterpType::Nearest,
            )
        } else {
            None
        }
    }

    fn print(&self, window: &Window) {
        let settings = PrintSettings::new();
        settings.set_quality(gtk::PrintQuality::High);
        settings.set_media_type(&"photographic");
        let print_operation = PrintOperation::new();
        print_operation.set_print_settings(Some(&settings));

        let original = self.original.clone();
        print_operation.connect_draw_page(move |_, print_context, _| {
            if let Some(image) = original.clone() {
                let image = &image
                    .scale_simple(
                        print_context.width() as i32,
                        print_context.height() as i32,
                        InterpType::Bilinear,
                    )
                    .expect("Unable to scale image");
                let cairo_context = print_context.cairo_context();
                cairo_context.set_source_pixbuf(
                    &image,
                    (print_context.width() - image.width() as f64) / 2.0,
                    (print_context.height() - image.height() as f64) / 2.0,
                );
                if let Err(error) = cairo_context.paint() {
                    tracing::error!("Couldn't print current image: {}", error);
                }
            }
        });

        print_operation.set_allow_async(true);
        print_operation
            .run(gtk::PrintOperationAction::PrintDialog, Some(window))
            .unwrap();
    }
}
