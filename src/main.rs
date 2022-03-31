// #![windows_subsystem="windows"]

mod download;
mod serial;
mod dataframe;
mod export;

use std::str::FromStr;
use dataframe::Dataframe;

use iced::{
    button, executor, Align, Application, Button, Column, Command,
    Element, Settings, Subscription, Text, Radio, Clipboard, Row, Length,
};

#[derive(Default)]
struct Reader {
    start_button: button::State,
    export_button: button::State,
    copy_button: button::State,
    port: i32,
    active: bool,
    last_value: Dataframe,
    snapshots: Vec<dataframe::Dataframe>,
}

#[derive(Debug, Clone)]
enum Message {
    RadioSelected(i32),
    SerialStartStop,
    SerialUpdate(download::Progress),
    Export,
    CopyToClipboard,
}

impl Application for Reader {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Bike Fitting")
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::RadioSelected(v) => {
                self.port = v;
            }
            Message::SerialStartStop => {
                self.active = !self.active;
            }
            Message::SerialUpdate(message) => {
                match message {
                    download::Progress::Started => {
                        // no op
                    }
                    download::Progress::Advanced(line) => {
                        println!("line {}", line);
                        match Dataframe::from_str(&line) {
                            Ok(frame) => {
                                if self.active {
                                    self.last_value = frame;
                                    if frame.action == 1 {
                                        self.snapshots.push(frame);
                                    }
                                }
                            },
                            Err(_e) => println!("SerialUpdate error")
                        }
                    }
                    download::Progress::Errored => {
                        // no op
                    }
                }
            }
            Message::Export => {
                match export::export_data(&self.snapshots) {
                    Ok(()) => println!("Export complete"),
                    Err(e) => eprint!("{:?}", e)
                }
            }
            Message::CopyToClipboard => {
                let mut payload = "".to_string();
                payload.push_str("ID\tX\tY\n");
                for (index, frame) in self.snapshots.iter().enumerate() {
                    payload.push_str(&(format!("{}\t{}\t{}\n", index + 1, frame.x, frame.y)));
                }
                clipboard.write(payload);
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.active {
            download::file(serial::port_name_of(self.port))
                .map(Message::SerialUpdate)
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let window = Row::new().padding(40).spacing(20).align_items(Align::Center);
        let mut ports = Column::new().spacing(20).align_items(Align::Start).width(Length::Fill).height(Length::Fill);
        let mut data = Column::new().spacing(20).align_items(Align::Start).width(Length::Fill).height(Length::Fill);
        let mut list = Column::new().spacing(10).align_items(Align::Start).width(Length::Fill).height(Length::Fill);

        ports = ports.push(
            Text::new("Portauswahl").size(30).height(Length::Units(50))
        );
        for p in serial::get_ports() {
            ports = ports.push(Radio::new(p.index, format!("{}", p.name), Some(self.port), Message::RadioSelected))
        }
        let label = if self.active { "Stop" } else { "Start" };
        ports = ports.push(
            Button::new(&mut self.start_button, Text::new(label)).on_press(Message::SerialStartStop)
        );

        data = data.push(
            Text::new("Live").size(30).height(Length::Units(50))
        )
        .push(
            Column::new().spacing(20).align_items(Align::Center).push(
                Row::new().spacing(12).align_items(Align::Center).push(
                    Text::new("X")
                ).push(
                    Text::new(self.last_value.x.to_string()).size(30)
                )
            ).push(
                Row::new().spacing(12).align_items(Align::Center).push(
                    Text::new("Y")
                ).push(
                    Text::new(self.last_value.y.to_string()).size(30)
                )
            )
        );

        list = list.push(
            Text::new("Daten").size(30).height(Length::Units(50))
        ).push(
            Button::new(&mut self.export_button, Text::new("Export")).on_press(Message::Export)
        ).push(
            Button::new(&mut self.copy_button, Text::new("Kopieren")).on_press(Message::CopyToClipboard)
        );
        let mut index = 1;
        for snapshot in &self.snapshots {
            let str = format!("{} {} {}", index, snapshot.x, snapshot.y);
            list = list.push(Text::new(str).width(Length::Fill));
            index += 1;
        }

        window.push(ports).push(data).push(list).into()
    }
}

fn main() {
    let result = Reader::run(Settings::default());
    match result {
        Ok(v) => println!("v {:?}", v),
        Err(e) => println!("e {:?}", e)
    }
}
