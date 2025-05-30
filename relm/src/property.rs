use gtk::{
    glib::{self},
    prelude::*,
    EntryBuffer, Frame, Label, SizeGroup,
};
use libindigo::{
    property::{PropertyData, PropertyItem, PropertyValue}, BlobItem, LightItem, NamedObject as _, NumberItem, Property, PropertyType, SwitchItem as _, TextItem
};
use log::warn;
use relm4::{
    factory::{FactoryHashMap, FactoryVecDequeBuilder, FactoryView},
    gtk,
    prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque},
    view, Component, ComponentParts, ComponentSender, FactorySender, RelmWidgetExt,
    SimpleComponent,
};

thread_local! {
    static PROP_COLUMN_1: SizeGroup = SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    static PROP_COLUMN_2: SizeGroup = SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    static PROP_COLUMN_3: SizeGroup = SizeGroup::new(gtk::SizeGroupMode::Horizontal);
}

#[derive(Debug)]
pub(crate) struct RelmProperty {
    property: PropertyData,
    items: FactoryVecDeque<RelmPropertyItem>,
}

#[derive(Debug, Clone)]
pub(crate) enum RelmPropertyInput {
    UpdateProperty(PropertyData),
}

#[derive(Debug, Clone)]
pub(crate) enum RelmPropertyOutput {
    RequestPropertyUpdate(PropertyData),
    RequestItemUpdate(PropertyItem),
}

#[relm4::factory(pub)]
impl FactoryComponent for RelmProperty {
    type Init = PropertyData;
    type Input = RelmPropertyInput;
    type Output = RelmPropertyOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = String;

    view! {
        #[root]
        gtk::Frame {
            set_label: Some(self.property.label()),
            self.items.widget() ->
            &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
            }
        }
    }

    fn init_model(property: Self::Init, _index: &String, _sender: FactorySender<Self>) -> Self {
        let items = FactoryVecDeque::builder()
            .launch(Self::ParentWidget::default())
            .detach();
        // .forward(sender.input_sender(), |output| match output {
        //     Prop => PropertyInput::,
        // });
        Self { property, items }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        for item in self.property.items() {
            self.items.guard().push_back(item.clone());
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            RelmPropertyInput::UpdateProperty(p) => self.property.update(&p),
        }
    }
}

#[derive(Debug, Clone)]
struct RelmPropertyItem {
    item: PropertyItem,
}

#[derive(Debug)]
pub enum PropertyItemInput {
    Toggle,
}

#[derive(Debug)]
pub enum PropertyItemOutput {
    Toggle,
}

#[relm4::factory]
impl FactoryComponent for RelmPropertyItem {
    type Init = PropertyItem;
    type Input = PropertyItemInput;
    type Output = PropertyItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            append = match &self.item.value() {
                PropertyValue::Text(text) => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_1.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: &self.item.label(),
                            },
                        },
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_2.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: text.value(),
                            }
                        }
                    }
                }
                // PropertyValue::Number{format, min, max, step, value, target} => {
                PropertyValue::Number(number) => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_1.with(|w| w.clone()),
                            gtk::Label {
                                #[watch]
                                set_label: &self.item.name(),
                            },
                        },
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_2.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: number.formatted_number().as_str(),
                            }
                        }
                    }                }
                PropertyValue::Light(_light) => {
                    gtk::Label {
                        set_label: "TODO: render light property"
                    }
                }
                PropertyValue::Switch(switch) => {
                    gtk::CheckButton {
                        #[watch]
                        set_active: switch.on(),
                        set_label: Some(&self.item.name()),
                        // TODO group checkbuttons depending on the property SwitchType...
                        // #[track(true)]
                        // set_group: Some("apa"),
                    }
                }
                // PropertyValue::Blob{format, url, size, value} => {
                PropertyValue::Blob(_blob) => {
                    gtk::Label {
                        set_label: "TODO: render blob property"
                    }
                }
            },
        }

    }

    fn init_model(item: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { item }
    }
}

impl RelmPropertyItem {
    fn label(&self) -> &str {
        &self.item.label()
    }

    fn new(item: PropertyItem) -> Self {
        Self { item }
    }
}

struct SwitchItem {
    item: PropertyItem,
}

#[derive(Debug)]
pub enum SwitchCommand {
    Toggle,
}

#[derive(Debug)]
pub enum SwitchStatus {
    On,
    Off,
}

#[relm4::factory]
impl FactoryComponent for SwitchItem {
    type Init = PropertyItem;
    type Input = SwitchCommand;
    type Output = SwitchStatus;
    type CommandOutput = ();
    type ParentWidget = gtk::Stack;

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
        self.item.label()
    }
    fn value(&self) -> bool {
        if let PropertyValue::Switch(switch) = self.item.value() {
            switch.on()
        } else {
            warn!("defaulting to 'off': expected switch, found {:?}", self.item.value());
            false
        }
    }
}
