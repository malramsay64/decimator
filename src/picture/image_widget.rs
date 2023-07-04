use gtk::prelude::*;
use relm4::component::{AsyncComponent, AsyncComponentParts};
use relm4::drawing::{DrawContext, DrawHandler};
use relm4::gtk::cairo::Operator;
use relm4::gtk::gdk_pixbuf::{Colorspace, InterpType, Pixbuf};
use relm4::gtk::prelude::GdkCairoContextExt;
use relm4::gtk::{EventControllerScrollFlags, Inhibit, PrintOperation, PrintSettings};
use relm4::safe_settings_and_actions::extensions::*;
use relm4::{gtk, AsyncComponentSender};
use relm4_icons::icon_name;

use super::ViewPreviewMsg;
use crate::Zoom;

#[derive(Debug, Default)]
pub struct ImageWidget {
    pub original: Option<Pixbuf>,
    pub preview: Option<Pixbuf>,
    view_width: u32,
    view_height: u32,
    zoom: f64,
    handler: DrawHandler,
}

#[derive(Debug)]
pub enum ZoomStates {
    Increase,
    Decrease,
    Fit,
    Native,
    Toggle(f64, f64),
}

#[derive(Debug)]
pub enum ImageMsg {
    Resize((u32, u32)),
    Scale(ZoomStates),
    SetImage(Option<Pixbuf>),
    UpdatePreview,
    Print(gtk::Window),
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
        let controller_zoom = gtk::EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);

        let zoom_sender = sender.clone();
        controller_zoom.connect_scroll(move |_, _, y| {
            println!("{y}");
            if y > 0. {
                zoom_sender.input(ImageMsg::Scale(ZoomStates::Increase));
            } else if y < 0. {
                zoom_sender.input(ImageMsg::Scale(ZoomStates::Decrease));
            }
            Inhibit(false)
        });

        let click_zoom = gtk::GestureClick::new();
        let zoom_sender = sender.clone();
        click_zoom.connect_pressed(move |_gesture, n_press, x, y| {
            if n_press == 2 {
                zoom_sender.input(ImageMsg::Scale(ZoomStates::Toggle(x, y)));
            }
        });
        area.add_controller(click_zoom);
        area.add_controller(controller_zoom);

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
                sender.input(ImageMsg::UpdatePreview);
            }
            ImageMsg::Scale(scale) => {
                let (x, y): (f64, f64) = match scale {
                    ZoomStates::Increase => {
                        self.zoom = self.zoom * 1.2;
                        if self.zoom > 200. {
                            self.zoom = 200.
                        }
                        (0., 0.)
                    }
                    ZoomStates::Decrease => {
                        self.zoom = self.zoom / 1.2;
                        if self.zoom < 10. {
                            self.zoom = 10.
                        }
                        (0., 0.)
                    }
                    ZoomStates::Native => {
                        self.zoom = 100.;
                        (0., 0.)
                    }
                    ZoomStates::Fit => {
                        self.zoom = self.scale_fit();
                        (0., 0.)
                    }
                    ZoomStates::Toggle(x, y) => {
                        if self.zoom == 100. {
                            self.zoom = self.scale_fit();
                            (0., 0.)
                        } else {
                            let prev_zoom = self.zoom as f64;
                            self.zoom = 100.;
                            let max_offset_x = (self.original.as_ref().unwrap().width() as f64
                                - self.view_width as f64)
                                / 2.;
                            let max_offset_y = (self.original.as_ref().unwrap().height() as f64
                                - self.view_height as f64)
                                / 2.;
                            let offset_x = -(x - self.view_width as f64 / 2.) * 100. / prev_zoom;
                            let offset_y = -(y - self.view_height as f64 / 2.) * 100. / prev_zoom;

                            (
                                offset_x.clamp(-max_offset_x, max_offset_x),
                                offset_y.clamp(-max_offset_y, max_offset_y),
                            )
                        }
                    }
                };
                self.view_scale(self.zoom, x, y);
                sender.input(ImageMsg::UpdatePreview);
            }
            ImageMsg::SetImage(image) => {
                self.original = image;
                self.zoom = self.scale_fit();
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
        // TODO: Handle a None preview
        if self.view_width == 0 && self.view_height == 0 {
            self.preview = self.original.clone();
        } else if self.original.is_some() {
            // tracing::info!("Resizing to {} x {}.", self.view_width, self.view_height);
            self.view_scale(self.zoom, 0., 0.);
        }
    }

    #[tracing::instrument(name = "Changing scale of image", level = "info", skip(self))]
    pub fn view_scale(&mut self, scale: f64, offset_x: f64, offset_y: f64) {
        tracing::info!("Zooming image to {scale:?}.");
        if let Some(i) = &self.original {
            let width = (i.width() as f64 * scale / 100.0) as i32;
            let height = (i.height() as f64 * scale / 100.0) as i32;
            tracing::debug!("Offsets: {offset_x} {offset_y}");
            let dest = Pixbuf::new(Colorspace::Rgb, true, 8, width, height)
                .expect("Unable to create new pixbuf");
            i.scale(
                &dest,
                0,
                0,
                width,
                height,
                offset_x,
                offset_y,
                scale as f64 / 100.0,
                scale as f64 / 100.0,
                InterpType::Bilinear,
            );

            self.preview = Some(dest);
        };
    }

    pub fn scale_fit(&self) -> f64 {
        if let Some(image) = &self.original {
            let image_width = image.width() as f64;
            let image_height = image.height() as f64;
            let width_ratio = self.view_width as f64 / image_width;
            let height_ratio = self.view_height as f64 / image_height;
            width_ratio.min(height_ratio) * 100.
        } else {
            100.
        }
    }

    fn print(&self, window: &gtk::Window) {
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
