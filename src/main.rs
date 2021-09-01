use std::{sync::mpsc, thread};

use anyhow::{anyhow, Result};
use cpal::{
    Device,
    traits::{DeviceTrait, HostTrait, StreamTrait}
};
use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};
use pitch_detection::{detector::mcleod::McLeodDetector, detector::PitchDetector};

mod moving_average;
mod ui;

use ui::{tui::TUI, Request, Response, UI};

fn main() -> Result<()> {
    let (tx_request, rx_request) = mpsc::channel();
    let (tx_response, rx_response) = mpsc::channel();

    let ui = TUI::new(rx_request, tx_response)?;

    thread::spawn(move || {
        tx_request
            .send(Request::GetHost(cpal::available_hosts()))
            .unwrap();

        let host = match rx_response.recv().unwrap() {
            Response::HostId(host_id) => cpal::host_from_id(host_id).unwrap(),
            _ => unreachable!(),
        };

        let mic_list = host
            .input_devices()
            .unwrap()
            .map(|dev| dev.name().unwrap())
            .collect::<Vec<_>>();
        tx_request.send(Request::GetMic(mic_list)).unwrap();

        let mic_device = match rx_response.recv().unwrap() {
            Response::MicName(mic_name) => host
                .input_devices()
                .unwrap()
                .find(|dev| dev.name().unwrap() == mic_name)
                .unwrap(),
            _ => unreachable!(),
        };

        let midi_output = MidiOutput::new("Voice2MIDI").unwrap();
        let ports = midi_output.ports().iter().cloned().map(|port| (midi_output.port_name(&port).unwrap(), port)).collect();

        tx_request
            .send(Request::GetMidiPort(ports))
            .unwrap();

        let output_port = match rx_response.recv().unwrap() {
            Response::MidiOutputPort(output_port) => output_port,
            _ => unreachable!(),
        };

        main_audio_loop(mic_device, midi_output, output_port).unwrap();
    });

    ui.run_main_loop();

    Ok(())
}

fn main_audio_loop(mic_device: Device, midi_output: MidiOutput, output_port: MidiOutputPort) -> Result<()> {
    let mic_config: cpal::StreamConfig = mic_device
        .supported_input_configs()?
        .next()
        .ok_or_else(|| anyhow!("Failed to find a supported mic input config!"))?
        .with_max_sample_rate()
        .into();

    // Keep a note of the sample rate
    let sample_rate = mic_config.sample_rate.0;

    // Create the MIDI output
    let mut conn_out = midi_output
        .connect(&output_port, "Keyboard")
        .map_err(|_| anyhow!("Failed to connect to MIDI port"))?;

    let mut last_note = 0;
    let mut average = moving_average::MovingAverage::new(50);

    let stream = mic_device.build_input_stream(
        &mic_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // react to stream events and read or write stream data here.
            let mut detector = McLeodDetector::new(data.len(), data.len() / 2);

            let pitch = detector.get_pitch(&data, sample_rate as usize, 5.0, 0.6);

            if let Some(pitch) = pitch {
                // println!("Frequency: {}, Clarity: {}", pitch.frequency, pitch.clarity);
                let midi_note = (12.0 * (pitch.frequency / 440.0).log2() + 69.0).trunc() as i32;

                let change_note =
                    // Has to be a valid MIDI note
                    (21..108).contains(&midi_note) &&
                    // Has to differ from the moving average to switch
                    (average.average() - midi_note).abs() > 2;

                if change_note {
                    note_off(&mut conn_out, last_note as u8);
                    last_note = midi_note;
                    note_on(&mut conn_out, midi_note as u8);
                    average.feed(last_note);
                }
            } else {
                note_off(&mut conn_out, last_note as u8);
                last_note = 0;
                average.feed(last_note);
            }
        },
        move |_err| {
            // react to errors here.
        },
    )?;

    stream.play()?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100000));
    }
}

const VELOCITY: u8 = 0x64;

fn note_on(out: &mut MidiOutputConnection, note: u8) {
    const NOTE_ON_MSG: u8 = 0x90;
    out.send(&[NOTE_ON_MSG, note, VELOCITY]).unwrap();
}

fn note_off(out: &mut MidiOutputConnection, note: u8) {
    const NOTE_OFF_MSG: u8 = 0x80;
    out.send(&[NOTE_OFF_MSG, note, VELOCITY]).unwrap();
}
