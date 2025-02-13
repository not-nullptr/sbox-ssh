use crossterm::{
    cursor::{self, MoveTo},
    style::{Color, SetBackgroundColor},
    terminal, QueueableCommand,
};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::{io, io::Write, path::PathBuf};
use video_rs::Decoder;

fn main() -> anyhow::Result<()> {
    // make sure we clean up terminal on exit
    crossterm::terminal::enable_raw_mode()?;
    std::panic::set_hook(Box::new(|_| {
        let _ = crossterm::terminal::disable_raw_mode();
    }));
    video_rs::init().map_err(|e| anyhow::anyhow!("{}", e))?;
    // read source
    let source = "video.mp4".parse::<PathBuf>()?;
    let mut decoder = Decoder::new(source)?;
    let fps = decoder.frame_rate();
    let sleep_time = std::time::Duration::from_secs_f64(1.0 / fps as f64);
    let mut stdout = io::stdout();
    stdout
        .queue(cursor::Hide)?
        .queue(terminal::Clear(terminal::ClearType::All))?;
    while let Ok(frame) = decoder.decode_raw() {
        let width = frame.width();
        let height = frame.height();
        let data = frame.data(0).to_vec();

        let img =
            DynamicImage::ImageRgb8(image::ImageBuffer::from_raw(width, height, data).unwrap());

        let (t_width, t_height) = terminal::size()?;

        // scale image so that it fits in terminal
        let scale = f64::min(
            t_width as f64 / width as f64,
            t_height as f64 / height as f64,
        );
        let width = (width as f64 * scale) as u32;
        let height = (height as f64 * scale) as u32;

        let resized = img.resize(width, height, FilterType::Nearest);

        stdout.queue(MoveTo(0, 0))?;

        for y in 0..resized.height() {
            for x in 0..resized.width() * 2 {
                let pixel = resized.get_pixel(x / 2, y);
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                stdout
                    .queue(MoveTo(x as u16, y as u16))?
                    .queue(SetBackgroundColor(Color::Rgb { r, g, b }))?;
                stdout.write_all(b" ")?;
            }
        }

        stdout.flush()?;
        std::thread::sleep(sleep_time);
    }
    Ok(())
}
