fn main() {
    use plotters::prelude::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::io::Write as IoWrite; // See https://doc.rust-lang.org/std/macro.writeln.html
    let background_color = &BLACK;
    let background_fill = background_color.filled();
    #[allow(unused_variables)]
    let transparent_color = background_color.mix(0.);
    let color0 = &WHITE;
    let color01 = color0.mix(0.1);
    let color02 = color0.mix(0.2);
    let color1 = &plotters::style::RGBColor(255, 192, 0);
    let color2 = &plotters::style::RGBColor(0, 176, 80);
    let color3 = &plotters::style::RGBColor(32, 56, 100);
    let color_vec = vec![color0, color1, color2, color2, color3];
    #[allow(unused_variables)]
    let fill0 = color0.filled();
    #[allow(unused_variables)]
    let fill01 = color01.filled();
    #[allow(unused_variables)]
    let fill2 = color02.filled();
    #[allow(unused_variables)]
    let fill1 = color1.filled();
    #[allow(unused_variables)]
    let fill2 = color2.filled();
    #[allow(unused_variables)]
    let fill3 = color3.filled();
    let x_label_area_size = 70;
    let y_label_area_size = 90;
    let text_size0 = 60;
    let text_size1 = 44;
    let text_size2 = 34;
    #[allow(unused_variables)]
    let background_text = ("Calibri", 1).into_font().color(background_color);
    #[allow(unused_variables)]
    let text0 = ("Calibri", text_size0).into_font().color(color0);
    let text1 = ("Calibri", text_size1).into_font().color(color0);
    let text2 = ("Calibri", text_size2).into_font().color(color0);
    use plotters::style::text_anchor::{HPos, Pos, VPos};
    let text2c = text2.pos(Pos::new(HPos::Center, VPos::Top));
    let figure_file_name = "fondos00.png";
    let figure_path = std::path::Path::new(&figure_file_name);
    if figure_path.exists() {
        panic!(
            "This program just tried to rewrite {}; please debug",
            figure_path.to_str().unwrap()
        );
    }
    let drawing_area = BitMapBackend::new(figure_path, (1920, 1080)).into_drawing_area();
    drawing_area.fill(background_color).unwrap();
    println!("Hello, world!");
}
