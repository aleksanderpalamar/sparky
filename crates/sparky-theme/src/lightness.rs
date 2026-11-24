use iced_core::Color;

struct Lab {
    l: f32,
    a: f32,
    b: f32,
}

pub fn lightness(color: Color) -> f32 {
    lab(color).l
}

pub fn apart(text: Color, ground: Color, extra: f32) -> Color {
    let mut shade = lab(text);
    shade.l = match shade.l >= lightness(ground) {
        true => (shade.l + extra).min(1.0),
        false => (shade.l - extra).max(0.0),
    };
    Color {
        a: text.a,
        ..rgb(shade)
    }
}

fn lab(color: Color) -> Lab {
    let [r, g, b] = [color.r, color.g, color.b].map(linear);
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    Lab {
        l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    }
}

fn rgb(shade: Lab) -> Color {
    let l = (shade.l + 0.396_337_78 * shade.a + 0.215_803_76 * shade.b).powi(3);
    let m = (shade.l - 0.105_561_346 * shade.a - 0.063_854_17 * shade.b).powi(3);
    let s = (shade.l - 0.089_484_18 * shade.a - 1.291_485_5 * shade.b).powi(3);
    Color::from_rgb(
        encode(4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s),
        encode(-1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s),
        encode(-0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s),
    )
}

fn linear(channel: f32) -> f32 {
    match channel <= 0.04045 {
        true => channel / 12.92,
        false => ((channel + 0.055) / 1.055).powf(2.4),
    }
}

fn encode(channel: f32) -> f32 {
    let channel = channel.clamp(0.0, 1.0);
    match channel <= 0.003_130_8 {
        true => channel * 12.92,
        false => 1.055 * channel.powf(1.0 / 2.4) - 0.055,
    }
}
