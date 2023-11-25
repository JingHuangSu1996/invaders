use std::{error::Error, io, time::{Duration, Instant}, sync::mpsc};
use crossterm::{terminal, cursor, ExecutableCommand, event::{self, Event, KeyCode}};
use invaders::render;
use rusty_audio::Audio;
use invaders::frame::Drawable;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();

    audio.add("explode", "assets/sounds/explode.wav");
    audio.add("lose", "assets/sounds/lose.wav");
    audio.add("move", "assets/sounds/move.wav");
    audio.add("pew", "assets/sounds/pew.wav");
    audio.add("startup", "assets/sounds/startup.wav");
    audio.add("win", "assets/sounds/win.wav");

    audio.play("startup");


    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;

    audio.wait();

    // Render loop in a seperate thread 

    let (render_transciver, render_receiver) = mpsc::channel();
    let render_handle = std::thread::spawn(move || {
        let mut last_frame = invaders::frame::new_frame();
        let mut stdout = io::stdout();

        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let curr_frame = match render_receiver.recv() {
                Ok(x) => x, 
                Err(_) => break,
            };

            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = invaders::player::Player::new();
    let mut instant = Instant::now();
    let mut invaders = invaders::invaders::Invaders::new();
    // Game loop
    'gameloop: loop {
        // Pre-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = invaders::frame::new_frame();
        // Input 
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    KeyCode::Left => {
                        player.move_left();
                        audio.play("move");
                    },
                    KeyCode::Right => {
                        player.move_right();
                        audio.play("move");
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    },
                    _ => {}
                }
            }
        }

        // Update
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }

        // Draw & render
        // player.draw(&mut curr_frame);
        // invaders.draw(&mut curr_frame);

        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        
        let _ = render_transciver.send(curr_frame)?;
        std::thread::sleep(Duration::from_millis(1));
    }


    // Clean up
    drop(render_transciver);
    render_handle.join().unwrap();
    stdout.execute(cursor::Show)?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
