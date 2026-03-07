use gtk::{
    glib::{self},
    prelude::*,
    EntryBuffer, Frame, Label, SizeGroup,
};

use libindigo_rs::{LightState, Property, PropertyItem, PropertyValue, SwitchState};

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
    property: Property,
    items: FactoryVecDeque<RelmPropertyItem>,
}

#[derive(Debug, Clone)]
pub(crate) enum RelmPropertyInput {
    UpdateProperty(Property),
}

#[derive(Debug, Clone)]
pub(crate) enum RelmPropertyOutput {
    RequestPropertyUpdate(Property),
    RequestItemUpdate(PropertyItem),
}

#[relm4::factory(pub)]
impl FactoryComponent for RelmProperty {
    type Init = Property;
    type Input = RelmPropertyInput;
    type Output = RelmPropertyOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = String;

    view! {
        #[root]
        gtk::Frame {
            set_label: Some(&self.property.label),
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

        for (name, item) in &self.property.items {
            self.items.guard().push_back(item.clone());
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            RelmPropertyInput::UpdateProperty(p) => {
                // Update the property data
                self.property = p;
                // TODO: Update individual items in the UI
            }
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
            append = match &self.item.value {
                PropertyValue::Text(text) => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_1.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: &self.item.label,
                            },
                        },
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_2.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: text.as_str(),
                            }
                        }
                    }
                }
                PropertyValue::Number{value, min, max, step, format} => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_1.with(|w| w.clone()),
                            gtk::Label {
                                #[watch]
                                set_label: &self.item.name,
                            },
                        },
                        gtk::Box {
                            set_size_group: &PROP_COLUMN_2.with(|w| w.clone() ),
                            gtk::Label {
                                #[watch]
                                set_label: &format!("{}", value),
                            }
                        }
                    }
                }
                PropertyValue::Light{state} => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Label {
                            #[watch]
                            set_label: &self.item.name,
                        },
                        gtk::Label {
                            #[watch]
                            set_label: &format!("{:?}", state),
                        }
                    }
                }
                PropertyValue::Switch{state} => {
                    gtk::CheckButton {
                        #[watch]
                        set_active: matches!(state, SwitchState::On),
                        set_label: Some(&self.item.name),
                    }
                }
                PropertyValue::Blob{data: _, format, size} => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Label {
                            #[watch]
                            set_label: &self.item.name,
                        },
                        gtk::Label {
                            #[watch]
                            set_label: &std::format!("BLOB ({} bytes, {})", size, format),
                        }
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
        &self.item.label
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
        &self.item.label
    }

    fn value(&self) -> bool {
        if let PropertyValue::Switch { state } = &self.item.value {
            matches!(state, SwitchState::On)
        } else {
            warn!(
                "defaulting to 'off': expected switch, found {:?}",
                self.item.value
            );
            false
        }
    }
}
