use crate::components::mixer::InputChannels;
use crate::components::submix::linking_tree::LinkingTree;
use crate::components::submix::monitor_tree::MonitorTree;
use crate::profile::Attribute;
use anyhow::Result;
use enum_map::EnumMap;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::collections::HashMap;
use std::io::Write;
use strum::{EnumProperty, IntoEnumIterator};

#[derive(Debug)]
pub struct SubMixer {
    submix_enabled: bool,
    volume_table: EnumMap<InputChannels, u8>,
    monitor_tree: MonitorTree,
    linking_tree: LinkingTree,
}

impl Default for SubMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl SubMixer {
    pub fn new() -> Self {
        Self {
            submix_enabled: false,
            volume_table: Default::default(),
            monitor_tree: Default::default(),
            linking_tree: Default::default(),
        }
    }

    pub fn parse_submixer(&mut self, attributes: &Vec<Attribute>) -> Result<()> {
        for attr in attributes {
            if attr.name == "submixMode" {
                self.submix_enabled = matches!(attr.value.as_str(), "1");
                continue;
            }

            if attr.name.ends_with("Level") {
                let mut found = false;

                // Get the String key..
                let channel = attr.name.as_str();
                let channel = &channel[0..channel.len() - 5];

                let value: u8 = attr.value.parse()?;

                // Find the channel from the Prefix..
                for volume in InputChannels::iter() {
                    if volume.get_str("Name").unwrap() == channel {
                        // Set the value..
                        self.volume_table[volume] = value;
                        found = true;
                    }
                }

                if !found {
                    println!("Unable to find Channel: {channel}");
                }
                continue;
            }
        }
        Ok(())
    }

    pub fn write_submixer<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("submixerTree");

        // Create the values..
        let mut attributes: HashMap<String, String> = HashMap::default();
        attributes.insert(
            String::from("submixMode"),
            (self.submix_enabled as u8).to_string(),
        );

        for volume in InputChannels::iter() {
            let key = format!("{}Level", volume.get_str("Name").unwrap());
            let value = format!("{}", self.volume_table[volume]);

            attributes.insert(key, value);
        }

        for (key, value) in &attributes {
            elem.push_attribute((key.as_str(), value.as_str()));
        }

        // Push our attributes, and prepare for additional tags..
        writer.write_event(Event::Start(elem))?;

        self.monitor_tree.write_monitor_tree(writer)?;
        self.linking_tree.write_linking_tree(writer)?;

        // We're done.
        writer.write_event(Event::End(BytesEnd::new("submixerTree")))?;

        Ok(())
    }

    pub fn parse_monitor(&mut self, attributes: &Vec<Attribute>) -> Result<()> {
        self.monitor_tree.parse_monitor_tree(attributes)
    }

    pub fn parse_linking(&mut self, attributes: &Vec<Attribute>) -> Result<()> {
        self.linking_tree.parse_links(attributes)
    }

    pub fn submix_enabled(&self) -> bool {
        self.submix_enabled
    }
    pub fn volume_table(&self) -> EnumMap<InputChannels, u8> {
        self.volume_table
    }
    pub fn linking_tree(&self) -> &LinkingTree {
        &self.linking_tree
    }
}
