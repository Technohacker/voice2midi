use std::sync::mpsc::{Receiver, Sender};

use anyhow::Result;
use cpal::HostId;
use midir::{
    MidiOutputPort,
};

pub mod tui;

pub enum Request {
    GetHost(Vec<HostId>),
    GetMic(Vec<String>),
    GetMidiPort(Vec<(String, MidiOutputPort)>),
}

pub enum Response {
    HostId(HostId),
    MicName(String),
    MidiOutputPort(MidiOutputPort)
}

pub type RequestChannel = Receiver<Request>;
pub type ResponseChannel = Sender<Response>;

pub trait UI {
    fn new(request_channel: RequestChannel, response_channel: ResponseChannel) -> Result<Box<Self>>;
    fn run_main_loop(self);
    fn get_host(&mut self, hosts: Vec<HostId>);
    fn get_mic(&mut self, mic_devices: Vec<String>);
    fn get_midi_port(&mut self, midi_output_ports: Vec<(String, MidiOutputPort)>);
}
