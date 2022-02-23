use std::{time::Duration};
use rodio::{OutputStream, Source, OutputStreamHandle};

use std::io::{stdin, stdout, Write};
use std::error::Error;

use midir::{MidiInput, Ignore};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    let mut oscillator = 
        WavetableOscillator::new(
            44100,
            WaveTable::sine(64)
    );
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid input port selected")?
        }
    };
    
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        if message[1] == 63 && message[0] == 156 {
            println!("GOT THE NOTE SIR!");
            // TODO: how to allocate this task to different thread and...
            play_note(&stream_handle, &oscillator);
        } else if message[1] == 63 && message[0] == 140 {
            // TODO: how to send "stop that!" to that thread
            println!("STOPPED THE NOTE SIR!");
        }
    }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

struct WaveTable(Vec<f32>);

impl WaveTable {
    fn sine(wave_table_size: usize) -> Self {
        let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);

        for n in 0..wave_table_size {
            wave_table.push(
                (2.0 * std::f32::consts::PI * n as f32 / wave_table_size as f32).sin()
            );
        }

        Self(wave_table)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> f32 {
        self.0[index]
    }
}
struct WavetableOscillator {
    sample_rate: u32,
    wave_table: WaveTable,
    index: f32,
    index_increment: f32,
}

impl WavetableOscillator {
    fn new(sample_rate: u32, wave_table: WaveTable) -> Self {
        WavetableOscillator {
            sample_rate,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
        }
    }

    fn set_freq(&mut self, frequency: f32) {
        self.index_increment = frequency * self.wave_table.len() as f32 / self.sample_rate as f32;
    }

    fn get_sample(&mut self) -> f32 {
        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;

        sample
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * self.wave_table.get(truncated_index)
        + next_index_weight * self.wave_table.get(next_index)
    }
}

impl Iterator for WavetableOscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.get_sample())
    }
}

impl Source for WavetableOscillator {
    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn play_note(stream_handle: &OutputStreamHandle, oscillator: &WavetableOscillator) {
    /*
        OK!
        So this needs to have the set_freq be based on key input.
        The amplitude on the velocity.
        and its playtime based on "await key released" but with ADSR release time?

        channel send and recv to wavetable so it can handle multi-input.
    */

    let _result = stream_handle.play_raw(oscillator.convert_samples());
}