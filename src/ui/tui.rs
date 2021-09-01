use anyhow::Result;
use cpal::HostId;
use midir::MidiOutputPort;

use cursive::{
    CursiveRunnable,
    CursiveRunner,
    align::Align,
    theme::{
        Color,
        BaseColor,
        PaletteColor
    },
    views::{
        Dialog,
        SelectView
    },
    traits::*
};

use super::{
    UI,
    RequestChannel,
    Request,
    ResponseChannel,
    Response
};

pub struct TUI {
    cursive: CursiveRunner<CursiveRunnable>,
    request_channel: Option<RequestChannel>,
    response_channel: ResponseChannel,
}

impl UI for TUI {
    fn new(request_channel: RequestChannel, response_channel: ResponseChannel) -> Result<Box<Self>> {
        let mut cursive = cursive::default();
        cursive.update_theme(|theme| {
            theme.palette[PaletteColor::Background] = Color::Dark(BaseColor::Black);
        });

        cursive.add_global_callback('q', |s| s.quit());
    
        Ok(Box::from(Self {
            request_channel: Some(request_channel),
            response_channel,
            cursive: cursive.into_runner()
        }))
    }

    fn run_main_loop(mut self) {
        self.cursive.refresh();

        let request_channel = self.request_channel.take().unwrap();

        while self.cursive.is_running() {
            for request in request_channel.try_iter() {
                match request {
                    Request::GetHost(host_ids) => self.get_host(host_ids),
                    Request::GetMic(devices) => self.get_mic(devices),
                    Request::GetMidiPort(midi_output) => self.get_midi_port(midi_output)
                }
            }
            self.cursive.refresh();
            self.cursive.step();
        }
    }

    fn get_host(&mut self, hosts: Vec<HostId>) {
        let response_channel = self.response_channel.clone();

        self.cursive.add_layer(
            Dialog::around(
                SelectView::new()
                    .with_all(hosts.into_iter().map(|host_id| (format!("{:?}", host_id), host_id)))
                    .on_submit(move |cursive, host_id| {
                        cursive.pop_layer();
                        response_channel.send(Response::HostId(*host_id)).unwrap();
                    })
                    .align(Align::center())
                    .min_width(5)
            )
            .title("Select your audio host")
        );
    }

    fn get_mic(&mut self, mic_devices: Vec<String>) {
        let response_channel = self.response_channel.clone();

        self.cursive.add_layer(
            Dialog::around(
                SelectView::new()
                    .with_all_str(mic_devices)
                    .on_submit(move |cursive, device: &String| {
                        cursive.pop_layer();
                        response_channel.send(Response::MicName(device.to_owned())).unwrap();
                    })
                    .align(Align::center())
                    .min_width(5)
            )
            .title("Select your microphone")
        );
    }

    fn get_midi_port(&mut self, midi_output_ports: Vec<(String, MidiOutputPort)>) {
        let response_channel = self.response_channel.clone();

        self.cursive.add_layer(
            Dialog::around(
                SelectView::new()
                    .with_all(midi_output_ports.into_iter())
                    .on_submit(move |cursive, port| {
                        cursive.pop_layer();
                        response_channel.send(Response::MidiOutputPort(port.clone())).unwrap();
                    })
                    .align(Align::center())
                    .min_width(5)
            )
            .title("Select your MIDI Port")
        );
    }
}