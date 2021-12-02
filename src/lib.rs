//
// (c) 2021 Sindre Fjermestad
//
// changed from Hubert's example https://gitlab.com/hfiguiere/midi-control/-/blob/07d3c5e6a34840814bb9d367c7aa428d5d0afa64/examples/arturia-blink.rs
// to work with my different Arturia controller.

extern crate midi_control;
extern crate midir;

use std::sync::mpsc::channel;
use std::thread;
use std::time;

use midi_control::consts;
use midi_control::sysex::USysExDecoder;
use midi_control::transport::MidiMessageSend;
use midi_control::vendor::arturia;
use midi_control::{MidiMessage, SysExEvent};

/// Print a message on error returned.
macro_rules! print_on_err {
    ($e:expr) => {
        if let Err(err) = $e {
            eprintln!(
                "{}:{} Error '{}': {}",
                file!(),
                line!(),
                stringify!($e),
                err
            );
        }
    };
}

/// String to look for when enumerating the MIDI devices
const ARTURIA_DEVICE: &str = "Arturia KeyStep 32";

fn find_port<T>(midi_io: &T) -> Option<T::Port>
where
    T: midir::MidiIO,
{
    let mut device_port: Option<T::Port> = None;
    for port in midi_io.ports() {
        if let Ok(port_name) = midi_io.port_name(&port) {
            if port_name.contains(ARTURIA_DEVICE) {
                device_port = Some(port);
                break;
            }
        }
    }
    device_port
}

fn main() {
    let midi_input = midir::MidiInput::new(ARTURIA_DEVICE).unwrap();

    let device_port = find_port(&midi_input);
    if device_port.is_none() {
        println!("Input device not found!");
        return;
    }

    let (sender, _receiver) = channel::<MidiMessage>();

    let device_port = device_port.unwrap();
    let _connect_in = midi_input.connect(
        &device_port,
        ARTURIA_DEVICE,
        move |timestamp, data, sender| {
            let msg = MidiMessage::from(data);
            println!("{}: received {:?} => {:?}", timestamp, data, msg);
            print_on_err!(sender.send(msg));
        },
        sender,
    );

    // receiver would be synth program. I guess?
    // or really Any program (visual perchance? ;) ;) )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_ports() {
        extern crate midir;

        use std::io::{stdin, stdout, Write};
        use std::error::Error;

        use midir::{MidiInput, MidiOutput, Ignore};

        fn main() {
            match run() {
                Ok(_) => (),
                Err(err) => println!("Error: {}", err)
            }
        }

        fn run() -> Result<(), Box<dyn Error>> {
            let mut midi_in = MidiInput::new("midir test input")?;
            midi_in.ignore(Ignore::None);
            let midi_out = MidiOutput::new("midir test output")?;

            let mut input = String::new();

            loop {
                println!("Available input ports:");
                for (i, p) in midi_in.ports().iter().enumerate() {
                    println!("{}: {}", i, midi_in.port_name(p)?);
                }
                
                println!("\nAvailable output ports:");
                for (i, p) in midi_out.ports().iter().enumerate() {
                    println!("{}: {}", i, midi_out.port_name(p)?);
                }

                // run in endless loop if "--loop" parameter is specified
                match ::std::env::args().nth(1) {
                    Some(ref arg) if arg == "--loop" => {}
                    _ => break
                }
                print!("\nPress <enter> to retry ...");
                stdout().flush()?;
                input.clear();
                stdin().read_line(&mut input)?;
                println!("\n");
            }
            
            Ok(())
        }

        main();
    }

    #[test]
    fn it_works() {
        main();
    }
}
