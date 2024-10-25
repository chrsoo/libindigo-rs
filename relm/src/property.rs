use gtk::{glib::{self}, prelude::*, EntryBuffer, Label};
use libindigo::{PropertyItem, PropertyValue};
use relm4::{
    gtk, prelude::{DynamicIndex, FactoryComponent}, view, Component, ComponentParts, ComponentSender, FactorySender, RelmWidgetExt, SimpleComponent
};

struct SwitchItem {
    item: PropertyItem,
}

#[derive(Debug)]
pub enum SwitchCommand {
    Toggle,
}

#[derive(Debug)]
pub enum SwitchStatus {
    On, Off
}

#[relm4::factory]
impl FactoryComponent for SwitchItem {
    type Init = PropertyItem;
    type Input = SwitchCommand;
    type Output = SwitchStatus;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            gtk::Switch {
                #[watch]
                set_active: self.value(),
            },
            gtk::Label {
                #[watch]
                set_label: &self.label(),
            }
        }
    }

    fn init_model(item: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { item }
    }

}

impl SwitchItem {
    fn label(&self) -> &str {
        &self.item.label
    }
    fn value(&self) -> bool {
        if let PropertyValue::Switch(b) = self.item.value {
            b
        } else {
            unreachable!("expected 'PropertyStatus::Switch' found '{}'", &self.item.value)
        }
    }
}