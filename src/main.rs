use macroquad::{audio::{self, play_sound_once, stop_sound}, prelude::*};

#[macroquad::main("Sprunki But in Rust")]
async fn main() {
    request_new_screen_size(800.0, 768.0);

    // -------------------------------------------------- FOR OREN -------------------------------------------------- \\
    let oren: Result<Image, macroquad::Error> = Image::from_file_with_format(include_bytes!("sprites/oren.png"), None);
    let oren_kick: audio::Sound = audio::load_sound_from_bytes(include_bytes!("Sounds/kick.wav")).await.unwrap();

    let img = oren.expect("failed to load image");
    let mut oren_texture = Texture2D::from_image(&img);


    let mut oren_singing: bool = true;
    let mut oren_exists: bool = true;
    // -------------------------------------------------- FOR OREN -------------------------------------------------- \\


    loop {
        clear_background(WHITE);
        draw_texture(&oren_texture, 0.0, 0.0, WHITE);
        



        // -------------------------------------------------- FOR OREN -------------------------------------------------- \\

        play_sound_once(&oren_kick);

       // -------------------------------------------------- FOR OREN -------------------------------------------------- \\







        // ----------------------------- INPUT CHECKERS ----------------------------- \\
            if is_key_pressed(KeyCode::O) {
                if oren_exists {
                    oren_texture = Texture2D::empty();      
                    stop_sound(&oren_kick);
                    toggle(&mut oren_singing);
                    toggle(&mut oren_exists);      
                } else {
                    oren_texture = Texture2D::from_image(&img);
                    draw_texture(&oren_texture, 0.0, 0.0, WHITE);
                    toggle(&mut oren_singing);
                    toggle(&mut oren_exists);
                }
            }

            if is_mouse_button_pressed(MouseButton::Left) {
                let (x, y) = mouse_position();
                let mut x_matches: bool = false;
                let mut y_matches: bool = false;


                if x >= 1.0 && x <= 219.0 && y >= 1.0 && y <= 240.0 {
                    println!("x and y matches");
                    toggle(&mut x_matches);
                    toggle(&mut y_matches);
                }
            }
        // ----------------------------- INPUT CHECKERS ----------------------------- \\







        next_frame().await
    }
}


fn toggle(x: &mut bool) {
    if *x {
        *x = false;
    } else {
        *x = true;
    }
}
