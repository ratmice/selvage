use crate::whisperer::*;
use kurbo::{Affine, Shape};
use std::io;

/// Unsure about this, perhaps it would be better
/// to implement ShapeWhisperer on `pdf_writer::Content`,
/// having the caller add it to a `Pdf`.
pub struct Pdf {
    pdf: pdf_writer::Pdf,
    content: pdf_writer::Content,
    tolerance: f64,
    size: kurbo::Size,
}
///
/// Writer for single page pdfs of unrasterized vector images
impl Pdf {
    pub fn new(size: kurbo::Size, tolerance: f64) -> Self {
        use pdf_writer::{Pdf, Ref};

        // Define some indirect reference ids we'll use.
        let catalog_id = Ref::new(1);
        let page_tree_id = Ref::new(2);
        let page_id = Ref::new(3);

        // Write a document catalog and a page tree with one A4 page that uses no resources.
        let mut pdf = Pdf::new();
        pdf.catalog(catalog_id).pages(page_tree_id);
        pdf.pages(page_tree_id).kids([page_id]).count(1);

        let mut content = pdf_writer::Content::new();
        content.transform(array_magic(
            (Affine::translate((0.0, size.height)) * Affine::FLIP_Y).as_coeffs(),
            |x| x as f32,
        ));

        Self {
            pdf,
            content,
            tolerance,
            size,
        }
    }

    pub fn write(mut self, mut writer: impl io::Write) -> io::Result<()> {
        use pdf_writer::Ref;
        let page_tree_id = Ref::new(2);
        let page_id = Ref::new(3);
        let contents_id = Ref::new(4);
        self.pdf.stream(contents_id, &self.content.finish());
        self.pdf
            .page(page_id)
            .parent(page_tree_id)
            .media_box(pdf_writer::Rect::new(
                0.0,
                0.0,
                self.size.width as f32,
                self.size.height as f32,
            ))
            .contents(contents_id)
            .resources();
        let _ = writer.write(&self.pdf.finish())?;
        Ok(())
    }
}

fn array_magic<T, U, const SZ: usize, F: Fn(T) -> U>(src: [T; SZ], f: F) -> [U; SZ]
where
    T: Copy,
{
    use std::mem::MaybeUninit;

    let dest: [U; SZ] = unsafe {
        let mut dest = MaybeUninit::uninit();
        // safety, for i in 0..SZ dest[i] = f(x) initializing the entire range.
        for (i, &x) in src.iter().enumerate() {
            (dest.as_mut_ptr() as *mut U).add(i).write(f(x));
        }
        dest.assume_init()
    };

    dest
}

impl SceneWhisperer for Pdf {
    fn apply_paint_op(
        &mut self,
        op: PaintOpRef<'_, '_>,
        transform: Affine,
        _brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.content.save_state();
        self.content
            .transform(array_magic(transform.as_coeffs(), |x| x as f32));
        if let Some(line) = shape.as_line() {
            self.content.move_to(line.p0.x as f32, line.p0.y as f32);
            self.content.line_to(line.p1.x as f32, line.p1.y as f32);
        } else if let Some(r) = shape.as_rect() {
            self.content
                .rect(r.x0 as f32, r.x0 as f32, r.x1 as f32, r.x1 as f32);
        } else {
            let path = shape.into_path(self.tolerance);
            for elem in path {
                match elem {
                    kurbo::PathEl::MoveTo(pt) => {
                        self.content.move_to(pt.x as f32, pt.y as f32);
                    }
                    kurbo::PathEl::LineTo(pt) => {
                        self.content.line_to(pt.x as f32, pt.y as f32);
                    }
                    kurbo::PathEl::CurveTo(a, b, c) => {
                        self.content.cubic_to(
                            a.x as f32, a.y as f32, b.x as f32, b.y as f32, c.x as f32, c.y as f32,
                        );
                    }
                    kurbo::PathEl::ClosePath => {
                        self.content.close_path();
                    }
                    kurbo::PathEl::QuadTo(_a, _b) => {
                        unimplemented!()
                    }
                }
            }
        }
        match op {
            PaintOpRef::Fill { style, brush } => {
                match brush {
                    peniko::BrushRef::Solid(x) => {
                        self.content
                            .set_fill_rgb(x.r as f32, x.g as f32, x.b as f32);
                    }
                    peniko::BrushRef::Gradient(_x) => {}
                    peniko::BrushRef::Image(_x) => {}
                }
                match style {
                    peniko::Fill::EvenOdd => {
                        self.content.fill_even_odd();
                    }
                    peniko::Fill::NonZero => {
                        self.content.fill_nonzero();
                    }
                }
            }
            PaintOpRef::Stroke { style, brush } => {
                match brush {
                    peniko::BrushRef::Solid(x) => {
                        self.content
                            .set_fill_rgb(x.r as f32, x.g as f32, x.b as f32);
                    }
                    peniko::BrushRef::Gradient(_x) => {}
                    peniko::BrushRef::Image(_x) => {}
                }
                self.content.set_line_join(match style.join {
                    kurbo::Join::Bevel => pdf_writer::types::LineJoinStyle::BevelJoin,
                    kurbo::Join::Round => pdf_writer::types::LineJoinStyle::RoundJoin,
                    kurbo::Join::Miter => pdf_writer::types::LineJoinStyle::MiterJoin,
                });
                self.content.set_line_width(style.width as f32);
                self.content.set_miter_limit(style.miter_limit as f32);
                self.content.stroke();
            }
            PaintOpRef::PushLayer { blend: _, alpha: _ } => {}
        }
        self.content.restore_state();
    }
    fn apply_paint_ops<'a, 'b, I>(
        &mut self,
        ops: I,
        transform: Affine,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) where
        I: IntoIterator<Item = PaintOpRef<'a, 'b>>,
    {
        for op in ops {
            self.apply_paint_op(op, transform, brush_transform, shape)
        }
    }
}
